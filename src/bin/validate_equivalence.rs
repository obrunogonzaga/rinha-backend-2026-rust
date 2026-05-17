use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rinha_backend_2026::index::{DIMS, Index, SCALE, SearchMode};
use rinha_backend_2026::vector::{Payload, vectorize};
use serde::Deserialize;

const DEFAULT_SAMPLE: usize = 512;
const DEFAULT_RANDOM_SAMPLE: usize = 5000;
const RANDOM_SEED: u64 = 0x5eed_51ce_a5c0_1d99;

#[derive(Deserialize)]
struct TestData {
    entries: Vec<TestEntry>,
}

#[derive(Deserialize)]
struct TestEntry {
    request: Payload,
    expected_approved: bool,
    expected_fraud_score: f32,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("validate_equivalence failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let data_dir: PathBuf = args.next().unwrap_or_else(|| "data".to_string()).into();
    let test_data: PathBuf = args
        .next()
        .unwrap_or_else(|| "test/test-data.json".to_string())
        .into();
    let sample = args
        .next()
        .map(|s| s.parse::<usize>().map_err(|e| format!("parse sample: {e}")))
        .transpose()?
        .unwrap_or(DEFAULT_SAMPLE);
    let random_sample = args
        .next()
        .map(|s| {
            s.parse::<usize>()
                .map_err(|e| format!("parse random_sample: {e}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_RANDOM_SAMPLE);

    let optimized = Index::load(&data_dir, SearchMode::VpTree)?;
    let brute = Index::load(&data_dir, SearchMode::BruteForce)?;
    let data: TestData = serde_json::from_slice(
        &std::fs::read(&test_data).map_err(|e| format!("read {}: {e}", test_data.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", test_data.display()))?;

    // `expected_*` is the official brute-force label/score for every payload.
    validate_expected_outputs(&optimized, &data.entries)?;
    validate_top5_sample(&optimized, &brute, &data.entries, sample)?;
    let refs = read_reference_vectors(&data_dir)?;
    validate_random_sample(&optimized, &brute, &refs, random_sample)?;
    eprintln!(
        "equivalence ok: entries={} top5_sample={} random_sample={} refs={}",
        data.entries.len(),
        sample.min(data.entries.len()),
        random_sample,
        refs.len(),
    );
    Ok(())
}

fn validate_expected_outputs(index: &Index, entries: &[TestEntry]) -> Result<(), String> {
    for (i, entry) in entries.iter().enumerate() {
        let q = vectorize(&entry.request);
        let d = index.score(&q);
        if d.approved != entry.expected_approved {
            return Err(format!(
                "entry {i}: approved {} != expected {}",
                d.approved, entry.expected_approved
            ));
        }
        if (d.fraud_score - entry.expected_fraud_score).abs() > 1e-6 {
            return Err(format!(
                "entry {i}: fraud_score {} != expected {}",
                d.fraud_score, entry.expected_fraud_score
            ));
        }
    }
    Ok(())
}

fn validate_top5_sample(
    optimized: &Index,
    brute: &Index,
    entries: &[TestEntry],
    sample: usize,
) -> Result<(), String> {
    if entries.is_empty() || sample == 0 {
        return Ok(());
    }
    let stride = (entries.len() / sample.max(1)).max(1);
    let mut checked = 0usize;
    for i in (0..entries.len()).step_by(stride).take(sample) {
        let q = vectorize(&entries[i].request);
        let optimized_top = optimized.top5(&q);
        let brute_top = brute.brute_force_top5(&q);
        if optimized_top != brute_top {
            return Err(format!("entry {i}: top5 mismatch"));
        }
        checked += 1;
    }
    if checked == 0 {
        return Err("top5 sample checked zero entries".into());
    }
    Ok(())
}

/// Deterministic differential fuzz: queries derived from the baked reference
/// vectors (exact, near-perturbed, far-perturbed) plus uniform-random points.
/// The VP-Tree top-5 must equal the brute-force top-5 for every query. With
/// integer-exact pruning this is a regression net, not the exactness proof.
fn validate_random_sample(
    optimized: &Index,
    brute: &Index,
    refs: &[[f32; DIMS]],
    n: usize,
) -> Result<(), String> {
    if refs.is_empty() || n == 0 {
        return Ok(());
    }
    let mut rng = Lcg::new(RANDOM_SEED);
    for i in 0..n {
        let base = &refs[(rng.next_u64() as usize) % refs.len()];
        let q = match rng.next_u64() % 4 {
            0 => *base,
            1 => perturb(base, 1e-4, &mut rng),
            2 => perturb(base, 0.05, &mut rng),
            _ => uniform(&mut rng),
        };
        if optimized.top5(&q) != brute.brute_force_top5(&q) {
            return Err(format!("random query {i}: top5 mismatch (q={q:?})"));
        }
    }
    Ok(())
}

fn perturb(base: &[f32; DIMS], delta: f32, rng: &mut Lcg) -> [f32; DIMS] {
    let mut q = *base;
    for v in &mut q {
        *v += (rng.next_unit() * 2.0 - 1.0) * delta;
    }
    q
}

fn uniform(rng: &mut Lcg) -> [f32; DIMS] {
    let mut q = [0.0f32; DIMS];
    for v in &mut q {
        *v = (rng.next_unit() * 2.0 - 1.0) * 3.5;
    }
    q
}

fn read_reference_vectors(data_dir: &Path) -> Result<Vec<[f32; DIMS]>, String> {
    let path = data_dir.join("refs.i16.bin");
    let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() % (DIMS * 2) != 0 {
        return Err(format!(
            "{} size {} not a multiple of {}",
            path.display(),
            bytes.len(),
            DIMS * 2
        ));
    }
    let count = bytes.len() / (DIMS * 2);
    let mut out = Vec::with_capacity(count);
    for c in bytes.chunks_exact(DIMS * 2) {
        let mut v = [0.0f32; DIMS];
        for (i, slot) in v.iter_mut().enumerate() {
            let q = i16::from_le_bytes([c[i * 2], c[i * 2 + 1]]);
            *slot = q as f32 / SCALE;
        }
        out.push(v);
    }
    Ok(out)
}

/// SplitMix64-style deterministic PRNG. No external crates: the sample must
/// reproduce bit-for-bit across machines and CI runs.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn next_unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}
