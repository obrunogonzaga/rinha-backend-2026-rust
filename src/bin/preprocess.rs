use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use flate2::read::GzDecoder;
use rinha_backend_2026::index::{DIMS, LABEL_FRAUD, LABEL_LEGIT, SCALE, write_vptree_artifacts};
use serde::Deserialize;

const SCALE_I32: i32 = 10_000;

#[derive(Deserialize)]
struct Record {
    vector: [f32; DIMS],
    label: String,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .unwrap_or_else(|| "resources/references.json.gz".to_string());
    let out_dir = args.next().unwrap_or_else(|| "data".to_string());

    if let Err(e) = run(Path::new(&input), Path::new(&out_dir)) {
        eprintln!("preprocess failed: {e}");
        process::exit(1);
    }

    // Skip destructor cleanup. When the preprocess binary is compiled with
    // target-cpu=x86-64-v3 (ADR-0002) and run under QEMU's TCG emulation (the
    // case when `docker buildx --platform linux/amd64` runs on a non-x86_64
    // host), dropping the ~3 M Vec<Record> after output is already flushed has
    // been observed to SIGSEGV in glibc free under AVX2-heavy memcpy emulation.
    // The output files are written before this point, so skipping cleanup is
    // safe; the process is short-lived and build-time only.
    process::exit(0);
}

fn run(input: &Path, out_dir: &Path) -> Result<(), String> {
    let start = Instant::now();
    fs::create_dir_all(out_dir)
        .map_err(|e| format!("create_dir_all {}: {e}", out_dir.display()))?;

    let file = File::open(input).map_err(|e| format!("open {}: {e}", input.display()))?;
    let reader = BufReader::with_capacity(1 << 20, GzDecoder::new(file));

    let records: Vec<Record> =
        serde_json::from_reader(reader).map_err(|e| format!("parse json: {e}"))?;
    let count = records.len();

    let refs_path = out_dir.join("refs.i16.bin");
    let labels_path = out_dir.join("labels.bin");
    let meta_path = out_dir.join("metadata.json");

    let refs = write_refs(&records, &refs_path)?;
    write_labels(&records, &labels_path)?;
    write_metadata(count, &meta_path)?;
    write_vptree_artifacts(&refs, out_dir)?;

    let elapsed_ms = start.elapsed().as_millis();
    eprintln!("preprocess: count={count} dims={DIMS} scale={SCALE_I32} elapsed_ms={elapsed_ms}");
    eprintln!(
        "  refs:     {} ({} bytes)",
        refs_path.display(),
        count * DIMS * 2
    );
    eprintln!("  labels:   {} ({} bytes)", labels_path.display(), count);
    eprintln!("  metadata: {}", meta_path.display());
    eprintln!("  vptree:   {}", out_dir.join("vptree.nodes.bin").display());
    Ok(())
}

fn write_refs(records: &[Record], path: &PathBuf) -> Result<Box<[i16]>, String> {
    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut w = BufWriter::with_capacity(1 << 20, file);
    let mut refs = Vec::with_capacity(records.len() * DIMS);
    let mut buf = [0u8; DIMS * 2];
    for r in records {
        for (i, &x) in r.vector.iter().enumerate() {
            let q = quantize(x);
            refs.push(q);
            let bytes = q.to_le_bytes();
            buf[i * 2] = bytes[0];
            buf[i * 2 + 1] = bytes[1];
        }
        w.write_all(&buf).map_err(|e| format!("write refs: {e}"))?;
    }
    w.flush().map_err(|e| format!("flush refs: {e}"))?;
    Ok(refs.into_boxed_slice())
}

fn write_labels(records: &[Record], path: &PathBuf) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut w = BufWriter::with_capacity(1 << 16, file);
    for r in records {
        let byte = label_byte(&r.label)?;
        w.write_all(&[byte])
            .map_err(|e| format!("write label: {e}"))?;
    }
    w.flush().map_err(|e| format!("flush labels: {e}"))?;
    Ok(())
}

fn write_metadata(count: usize, path: &PathBuf) -> Result<(), String> {
    let meta = serde_json::json!({
        "count": count,
        "dims": DIMS,
        "scale": SCALE_I32,
        "quantization": "i16_round_half_away_from_zero",
        "endianness": "little",
        "stride_bytes": DIMS * 2,
        "label_encoding": { "legit": LABEL_LEGIT, "fraud": LABEL_FRAUD },
    });
    let mut s = serde_json::to_string_pretty(&meta).map_err(|e| format!("encode meta: {e}"))?;
    s.push('\n');
    fs::write(path, s).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}

fn quantize(v: f32) -> i16 {
    let scaled = (v * SCALE).round();
    if scaled >= i16::MAX as f32 {
        i16::MAX
    } else if scaled <= i16::MIN as f32 {
        i16::MIN
    } else {
        scaled as i16
    }
}

fn label_byte(s: &str) -> Result<u8, String> {
    match s {
        "legit" => Ok(LABEL_LEGIT),
        "fraud" => Ok(LABEL_FRAUD),
        other => Err(format!("unknown label: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dequantize(q: i16) -> f32 {
        q as f32 / SCALE
    }

    #[test]
    fn quantize_zero_and_one() {
        assert_eq!(quantize(0.0), 0);
        assert_eq!(quantize(1.0), 10_000);
    }

    #[test]
    fn quantize_minus_one_sentinel() {
        assert_eq!(quantize(-1.0), -10_000);
    }

    #[test]
    fn quantize_clamps_extreme_inputs_to_i16_bounds() {
        assert_eq!(quantize(1e6), i16::MAX);
        assert_eq!(quantize(-1e6), i16::MIN);
        assert_eq!(quantize(f32::INFINITY), i16::MAX);
        assert_eq!(quantize(f32::NEG_INFINITY), i16::MIN);
    }

    #[test]
    fn roundtrip_error_bounded_by_inverse_scale() {
        let inputs = [
            0.0_f32, 0.0001, 0.0041, 0.0833, 0.1667, 0.3333, 0.5, 0.6667, 0.7826, 0.9506, 1.0, -1.0,
        ];
        let max_err = 1.0 / SCALE;
        for x in inputs {
            let r = dequantize(quantize(x));
            assert!(
                (r - x).abs() <= max_err,
                "roundtrip x={x} got {r} err={}",
                (r - x).abs()
            );
        }
    }

    #[test]
    fn quantize_uses_round_half_away_from_zero() {
        assert_eq!(quantize(0.00005), 1);
        assert_eq!(quantize(-0.00005), -1);
        assert_eq!(quantize(0.99995), 10_000);
    }

    #[test]
    fn label_byte_known_labels() {
        assert_eq!(label_byte("legit").unwrap(), 0);
        assert_eq!(label_byte("fraud").unwrap(), 1);
    }

    #[test]
    fn label_byte_unknown_label_errors() {
        assert!(label_byte("other").is_err());
        assert!(label_byte("").is_err());
    }

    #[test]
    fn end_to_end_two_record_payload_produces_expected_bytes() {
        use std::io::Write;

        let json = r#"[
            {"vector":[0.01,0.0833,0.05,0.8261,0.1667,-1,-1,0.0432,0.25,0,1,0,0.2,0.0416],"label":"legit"},
            {"vector":[0.9506,0.8333,1.0,0.2174,0.8333,-1,-1,0.9523,1.0,0,1,1,0.75,0.0055],"label":"fraud"}
        ]"#;
        let tmp = std::env::temp_dir().join("rinha_2b_test_input.json.gz");
        let out = std::env::temp_dir().join("rinha_2b_test_out");

        {
            let f = File::create(&tmp).unwrap();
            let mut gz = flate2::write::GzEncoder::new(f, flate2::Compression::default());
            gz.write_all(json.as_bytes()).unwrap();
            gz.finish().unwrap();
        }
        let _ = fs::remove_dir_all(&out);
        run(&tmp, &out).unwrap();

        let refs = fs::read(out.join("refs.i16.bin")).unwrap();
        assert_eq!(refs.len(), 2 * DIMS * 2);
        let dim0_rec0 = i16::from_le_bytes([refs[0], refs[1]]);
        assert_eq!(dim0_rec0, 100); // 0.01 * 10000
        let dim5_rec0 = i16::from_le_bytes([refs[10], refs[11]]);
        assert_eq!(dim5_rec0, -10_000); // -1 sentinel

        let labels = fs::read(out.join("labels.bin")).unwrap();
        assert_eq!(labels, vec![LABEL_LEGIT, LABEL_FRAUD]);

        let meta: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("metadata.json")).unwrap()).unwrap();
        assert_eq!(meta["count"], 2);
        assert_eq!(meta["dims"], 14);
        assert_eq!(meta["scale"], 10_000);

        // Determinism: running twice produces byte-identical output.
        let refs1 = fs::read(out.join("refs.i16.bin")).unwrap();
        let labels1 = fs::read(out.join("labels.bin")).unwrap();
        run(&tmp, &out).unwrap();
        let refs2 = fs::read(out.join("refs.i16.bin")).unwrap();
        let labels2 = fs::read(out.join("labels.bin")).unwrap();
        assert_eq!(refs1, refs2);
        assert_eq!(labels1, labels2);

        let _ = fs::remove_file(&tmp);
        let _ = fs::remove_dir_all(&out);
    }
}
