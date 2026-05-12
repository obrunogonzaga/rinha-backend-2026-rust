use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

pub const DIMS: usize = 14;
pub const SCALE: f32 = 10_000.0;
pub const K: usize = 5;
pub const THRESHOLD: f32 = 0.6;

#[cfg_attr(not(test), allow(dead_code))]
pub const LABEL_LEGIT: u8 = 0;
pub const LABEL_FRAUD: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub approved: bool,
    pub fraud_score: f32,
}

pub struct Index {
    refs: RefStorage,
    labels: LabelStorage,
    count: usize,
}

enum RefStorage {
    Mmapped(Mmap),
    #[cfg_attr(not(test), allow(dead_code))]
    Owned(Box<[i16]>),
}

enum LabelStorage {
    Mmapped(Mmap),
    #[cfg_attr(not(test), allow(dead_code))]
    Owned(Box<[u8]>),
}

impl Index {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let refs_path = dir.join("refs.i16.bin");
        let labels_path = dir.join("labels.bin");

        let refs_file =
            File::open(&refs_path).map_err(|e| format!("open {}: {e}", refs_path.display()))?;
        let labels_file =
            File::open(&labels_path).map_err(|e| format!("open {}: {e}", labels_path.display()))?;

        let refs = unsafe { Mmap::map(&refs_file) }
            .map_err(|e| format!("mmap {}: {e}", refs_path.display()))?;
        let labels = unsafe { Mmap::map(&labels_file) }
            .map_err(|e| format!("mmap {}: {e}", labels_path.display()))?;

        let refs_len = refs.len();
        let labels_len = labels.len();
        if refs_len % (DIMS * 2) != 0 {
            return Err(format!(
                "refs.i16.bin size {refs_len} not a multiple of {} (dims * 2)",
                DIMS * 2
            ));
        }
        let count = refs_len / (DIMS * 2);
        if labels_len != count {
            return Err(format!(
                "labels.bin size {labels_len} != refs count {count}"
            ));
        }
        if count == 0 {
            return Err("empty index".into());
        }

        Ok(Self {
            refs: RefStorage::Mmapped(refs),
            labels: LabelStorage::Mmapped(labels),
            count,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_parts(refs: Box<[i16]>, labels: Box<[u8]>) -> Result<Self, String> {
        if refs.len() % DIMS != 0 {
            return Err(format!("refs len {} not a multiple of {DIMS}", refs.len()));
        }
        let count = refs.len() / DIMS;
        if labels.len() != count {
            return Err(format!("labels len {} != count {count}", labels.len()));
        }
        if count == 0 {
            return Err("empty index".into());
        }
        Ok(Self {
            refs: RefStorage::Owned(refs),
            labels: LabelStorage::Owned(labels),
            count,
        })
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn score(&self, q: &[f32; DIMS]) -> Decision {
        let qi = quantize_query(q);
        let refs = self.refs_as_i16();
        let labels = self.labels_as_u8();

        let mut top_dist = [i64::MAX; K];
        let mut top_lbl = [0u8; K];
        let mut max_idx: usize = 0;

        for (idx, chunk) in refs.chunks_exact(DIMS).enumerate() {
            let d = sq_dist_14(&qi, chunk);
            if d < top_dist[max_idx] {
                top_dist[max_idx] = d;
                top_lbl[max_idx] = labels[idx];
                max_idx = arg_max(&top_dist);
            }
        }

        let fraud_count = top_lbl.iter().filter(|&&l| l == LABEL_FRAUD).count();
        let fraud_score = fraud_count as f32 / K as f32;
        Decision {
            approved: fraud_score < THRESHOLD,
            fraud_score,
        }
    }

    fn refs_as_i16(&self) -> &[i16] {
        match &self.refs {
            RefStorage::Mmapped(m) => {
                let bytes: &[u8] = &m[..];
                let ptr = bytes.as_ptr() as *const i16;
                let len = bytes.len() / 2;
                debug_assert_eq!(ptr.align_offset(std::mem::align_of::<i16>()), 0);
                unsafe { std::slice::from_raw_parts(ptr, len) }
            }
            RefStorage::Owned(b) => b,
        }
    }

    fn labels_as_u8(&self) -> &[u8] {
        match &self.labels {
            LabelStorage::Mmapped(m) => &m[..],
            LabelStorage::Owned(b) => b,
        }
    }
}

#[inline]
fn sq_dist_14(q: &[i16; DIMS], r: &[i16]) -> i64 {
    let mut acc: i64 = 0;
    for i in 0..DIMS {
        let dx = q[i] as i32 - r[i] as i32;
        acc += (dx as i64) * (dx as i64);
    }
    acc
}

#[inline]
fn arg_max(a: &[i64; K]) -> usize {
    let mut m = 0usize;
    for i in 1..K {
        if a[i] > a[m] {
            m = i;
        }
    }
    m
}

#[inline]
fn quantize_query(q: &[f32; DIMS]) -> [i16; DIMS] {
    let mut out = [0i16; DIMS];
    for i in 0..DIMS {
        let scaled = (q[i] * SCALE).round();
        out[i] = if scaled >= i16::MAX as f32 {
            i16::MAX
        } else if scaled <= i16::MIN as f32 {
            i16::MIN
        } else {
            scaled as i16
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_index(refs: Vec<i16>, labels: Vec<u8>) -> Index {
        Index::from_parts(refs.into_boxed_slice(), labels.into_boxed_slice()).unwrap()
    }

    fn zeros() -> [f32; DIMS] {
        [0.0; DIMS]
    }

    #[test]
    fn quantize_query_matches_preprocess_quantize_for_known_values() {
        let q = [
            0.01_f32, 0.0833, 0.05, 0.8261, 0.1667, -1.0, -1.0, 0.0432, 0.25, 0.0, 1.0, 0.0, 0.2,
            0.0416,
        ];
        let qi = quantize_query(&q);
        assert_eq!(
            qi,
            [
                100i16, 833, 500, 8261, 1667, -10000, -10000, 432, 2500, 0, 10000, 0, 2000, 416
            ]
        );
    }

    #[test]
    fn sq_dist_zero_for_identical_vectors() {
        let q = [100i16; DIMS];
        let r = [100i16; DIMS];
        assert_eq!(sq_dist_14(&q, &r), 0);
    }

    #[test]
    fn sq_dist_handles_negative_sentinel() {
        let q = [-10000i16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let r = [10000i16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        // dx = -20000, dx^2 = 400_000_000
        assert_eq!(sq_dist_14(&q, &r), 400_000_000);
    }

    #[test]
    fn arg_max_returns_index_of_largest() {
        assert_eq!(arg_max(&[1, 2, 3, 4, 5]), 4);
        assert_eq!(arg_max(&[9, 2, 3, 4, 5]), 0);
        assert_eq!(arg_max(&[1, 9, 3, 4, 5]), 1);
    }

    #[test]
    fn score_all_fraud_returns_one_and_rejects() {
        let mut refs = Vec::with_capacity(5 * DIMS);
        for _ in 0..5 {
            refs.extend_from_slice(&[0i16; DIMS]);
        }
        let labels = vec![LABEL_FRAUD; 5];
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 1.0);
        assert!(!d.approved);
    }

    #[test]
    fn score_zero_fraud_returns_zero_and_approves() {
        let mut refs = Vec::with_capacity(5 * DIMS);
        for _ in 0..5 {
            refs.extend_from_slice(&[0i16; DIMS]);
        }
        let labels = vec![LABEL_LEGIT; 5];
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 0.0);
        assert!(d.approved);
    }

    #[test]
    fn score_three_fraud_returns_0_6_and_rejects() {
        let mut refs = Vec::with_capacity(5 * DIMS);
        for _ in 0..5 {
            refs.extend_from_slice(&[0i16; DIMS]);
        }
        let labels = vec![
            LABEL_FRAUD,
            LABEL_FRAUD,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_LEGIT,
        ];
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 0.6);
        assert!(!d.approved);
    }

    #[test]
    fn score_two_fraud_returns_0_4_and_approves() {
        let mut refs = Vec::with_capacity(5 * DIMS);
        for _ in 0..5 {
            refs.extend_from_slice(&[0i16; DIMS]);
        }
        let labels = vec![
            LABEL_FRAUD,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_LEGIT,
            LABEL_LEGIT,
        ];
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 0.4);
        assert!(d.approved);
    }

    #[test]
    fn top5_picks_nearest_when_more_than_five_refs() {
        // Build 7 refs: 5 close-to-zero legit, 2 far-away fraud.
        // Query at origin; the nearest 5 should be the legit ones, score 0.
        let mut refs: Vec<i16> = Vec::new();
        for i in 0..5 {
            let mut v = [0i16; DIMS];
            v[0] = i as i16; // tiny distances
            refs.extend_from_slice(&v);
        }
        for _ in 0..2 {
            let mut v = [0i16; DIMS];
            v[0] = 10_000; // far
            refs.extend_from_slice(&v);
        }
        let mut labels = vec![LABEL_LEGIT; 5];
        labels.extend_from_slice(&[LABEL_FRAUD, LABEL_FRAUD]);
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 0.0);
        assert!(d.approved);
    }

    #[test]
    fn top5_picks_fraud_when_query_close_to_fraud_cluster() {
        // 5 fraud near zero, 5 legit far away. Query at zero → fraud_score=1.
        let mut refs: Vec<i16> = Vec::new();
        for i in 0..5 {
            let mut v = [0i16; DIMS];
            v[0] = i as i16;
            refs.extend_from_slice(&v);
        }
        for _ in 0..5 {
            let mut v = [0i16; DIMS];
            v[0] = 10_000;
            refs.extend_from_slice(&v);
        }
        let mut labels = vec![LABEL_FRAUD; 5];
        labels.extend_from_slice(&[LABEL_LEGIT; 5]);
        let idx = build_index(refs, labels);
        let d = idx.score(&zeros());
        assert_eq!(d.fraud_score, 1.0);
        assert!(!d.approved);
    }

    #[test]
    fn from_parts_rejects_mismatched_lengths() {
        let refs = vec![0i16; DIMS * 2].into_boxed_slice();
        let labels = vec![LABEL_LEGIT; 3].into_boxed_slice();
        assert!(Index::from_parts(refs, labels).is_err());
    }

    #[test]
    fn from_parts_rejects_empty() {
        let refs = vec![0i16; 0].into_boxed_slice();
        let labels = vec![0u8; 0].into_boxed_slice();
        assert!(Index::from_parts(refs, labels).is_err());
    }
}
