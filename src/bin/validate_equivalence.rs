use std::path::PathBuf;
use std::process::ExitCode;

use rinha_backend_2026::index::{Index, SearchMode};
use rinha_backend_2026::vector::{Payload, vectorize};
use serde::Deserialize;

const DEFAULT_SAMPLE: usize = 512;

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

    let optimized = Index::load(&data_dir, SearchMode::VpTree)?;
    let brute = Index::load(&data_dir, SearchMode::BruteForce)?;
    let data: TestData = serde_json::from_slice(
        &std::fs::read(&test_data).map_err(|e| format!("read {}: {e}", test_data.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", test_data.display()))?;

    // `expected_*` is the official brute-force label/score for every payload.
    validate_expected_outputs(&optimized, &data.entries)?;
    validate_top5_sample(&optimized, &brute, &data.entries, sample)?;
    eprintln!(
        "equivalence ok: entries={} top5_sample={}",
        data.entries.len(),
        sample.min(data.entries.len())
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
