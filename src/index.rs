use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use memmap2::Mmap;
use serde::Deserialize;

pub const DIMS: usize = 14;
pub const SCALE: f32 = 10_000.0;
pub const K: usize = 5;
pub const THRESHOLD: f32 = 0.6;
pub const VPTREE_NODES_FILE: &str = "vptree.nodes.bin";
pub const VPTREE_METADATA_FILE: &str = "vptree.metadata.json";

#[cfg_attr(not(test), allow(dead_code))]
pub const LABEL_LEGIT: u8 = 0;
pub const LABEL_FRAUD: u8 = 1;
const NONE: u32 = u32::MAX;
const VPTREE_NODE_SIZE: usize = 20;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    VpTree,
    BruteForce,
}

impl SearchMode {
    pub fn from_env() -> Result<Self, String> {
        match std::env::var("SEARCH_MODE") {
            Ok(s) if s == "vptree" => Ok(Self::VpTree),
            Ok(s) if s == "bruteforce" || s == "brute_force" => Ok(Self::BruteForce),
            Ok(s) => Err(format!("unknown SEARCH_MODE={s}")),
            Err(std::env::VarError::NotPresent) => Ok(Self::VpTree),
            Err(e) => Err(format!("read SEARCH_MODE: {e}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbor {
    pub distance: u64,
    pub index: u32,
    pub label: u8,
}

impl Neighbor {
    fn worst() -> Self {
        Self {
            distance: u64::MAX,
            index: u32::MAX,
            label: LABEL_LEGIT,
        }
    }

    fn better_than(self, other: Self) -> bool {
        self.distance < other.distance
            || (self.distance == other.distance && self.index < other.index)
    }

    fn worse_than(self, other: Self) -> bool {
        self.distance > other.distance
            || (self.distance == other.distance && self.index > other.index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decision {
    pub approved: bool,
    pub fraud_score: f32,
}

pub struct Index {
    refs: RefStorage,
    labels: LabelStorage,
    count: usize,
    search: SearchIndex,
}

enum SearchIndex {
    BruteForce,
    VpTree(VpTree),
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
    pub fn load(dir: &Path, mode: SearchMode) -> Result<Self, String> {
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
        if count < K {
            return Err(format!("refs count {count} < k {K}"));
        }
        let search = match mode {
            SearchMode::BruteForce => SearchIndex::BruteForce,
            SearchMode::VpTree => SearchIndex::VpTree(VpTree::load(dir, count)?),
        };

        Ok(Self {
            refs: RefStorage::Mmapped(refs),
            labels: LabelStorage::Mmapped(labels),
            count,
            search,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_parts(refs: Box<[i16]>, labels: Box<[u8]>) -> Result<Self, String> {
        Self::from_parts_with_mode(refs, labels, SearchMode::BruteForce)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_parts_with_mode(
        refs: Box<[i16]>,
        labels: Box<[u8]>,
        mode: SearchMode,
    ) -> Result<Self, String> {
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
        if count < K {
            return Err(format!("refs count {count} < k {K}"));
        }
        let search = match mode {
            SearchMode::BruteForce => SearchIndex::BruteForce,
            SearchMode::VpTree => SearchIndex::VpTree(VpTree::build(&refs, count)?),
        };
        Ok(Self {
            refs: RefStorage::Owned(refs),
            labels: LabelStorage::Owned(labels),
            count,
            search,
        })
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn score(&self, q: &[f32; DIMS]) -> Decision {
        let top = self.top5(q);
        let fraud_count = top.iter().filter(|n| n.label == LABEL_FRAUD).count();
        let fraud_score = fraud_count as f32 / K as f32;
        Decision {
            approved: fraud_score < THRESHOLD,
            fraud_score,
        }
    }

    pub fn top5(&self, q: &[f32; DIMS]) -> [Neighbor; K] {
        let qi = quantize_query(q);
        match &self.search {
            SearchIndex::BruteForce => self.brute_force_top5_quantized(&qi),
            SearchIndex::VpTree(tree) => tree.search(self.refs_as_i16(), self.labels_as_u8(), &qi),
        }
    }

    pub fn brute_force_top5(&self, q: &[f32; DIMS]) -> [Neighbor; K] {
        let qi = quantize_query(q);
        self.brute_force_top5_quantized(&qi)
    }

    fn brute_force_top5_quantized(&self, q: &[i16; DIMS]) -> [Neighbor; K] {
        let refs = self.refs_as_i16();
        let labels = self.labels_as_u8();
        let mut top = TopK::new();
        for (idx, chunk) in refs.chunks_exact(DIMS).enumerate() {
            top.push(Neighbor {
                distance: sq_dist_14(q, chunk),
                index: idx as u32,
                label: labels[idx],
            });
        }
        top.into_sorted()
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

pub fn write_vptree_artifacts(refs: &[i16], out_dir: &Path) -> Result<(), String> {
    if refs.len() % DIMS != 0 {
        return Err(format!("refs len {} not a multiple of {DIMS}", refs.len()));
    }
    let count = refs.len() / DIMS;
    if count == 0 {
        return Err("empty index".into());
    }
    let tree = VpTree::build(refs, count)?;
    tree.write(out_dir, count)
}

struct TopK {
    items: [Neighbor; K],
    len: usize,
    worst_idx: usize,
}

impl TopK {
    fn new() -> Self {
        Self {
            items: [Neighbor::worst(); K],
            len: 0,
            worst_idx: 0,
        }
    }

    fn push(&mut self, n: Neighbor) {
        if self.len < K {
            self.items[self.len] = n;
            self.len += 1;
            self.recompute_worst();
            return;
        }
        if n.better_than(self.items[self.worst_idx]) {
            self.items[self.worst_idx] = n;
            self.recompute_worst();
        }
    }

    fn tau_sq(&self) -> u64 {
        if self.len < K {
            u64::MAX
        } else {
            self.items[self.worst_idx].distance
        }
    }

    fn into_sorted(mut self) -> [Neighbor; K] {
        self.items
            .sort_by(|a, b| a.distance.cmp(&b.distance).then(a.index.cmp(&b.index)));
        self.items
    }

    fn recompute_worst(&mut self) {
        let mut idx = 0;
        for i in 1..K {
            if self.items[i].worse_than(self.items[idx]) {
                idx = i;
            }
        }
        self.worst_idx = idx;
    }
}

struct VpTree {
    nodes: VpNodeStorage,
}

enum VpNodeStorage {
    Mmapped(Mmap),
    Owned(Box<[VpNode]>),
}

#[derive(Clone, Copy)]
struct VpNode {
    point_index: u32,
    threshold_sq: u64,
    left: u32,
    right: u32,
}

#[derive(Deserialize)]
struct VpTreeMetadata {
    count: usize,
    node_count: usize,
    dims: usize,
    node_size: usize,
    root: u32,
    checksum64: String,
}

impl VpTree {
    fn build(refs: &[i16], count: usize) -> Result<Self, String> {
        let mut ids: Vec<u32> = (0..count)
            .map(|i| u32::try_from(i).map_err(|_| format!("index {i} exceeds u32")))
            .collect::<Result<_, _>>()?;
        let mut nodes = Vec::with_capacity(count);
        build_vptree_node(refs, &mut ids, &mut nodes);
        Ok(Self {
            nodes: VpNodeStorage::Owned(nodes.into_boxed_slice()),
        })
    }

    fn load(dir: &Path, count: usize) -> Result<Self, String> {
        let meta_path = dir.join(VPTREE_METADATA_FILE);
        let nodes_path = dir.join(VPTREE_NODES_FILE);
        let meta: VpTreeMetadata = serde_json::from_slice(
            &fs::read(&meta_path).map_err(|e| format!("read {}: {e}", meta_path.display()))?,
        )
        .map_err(|e| format!("parse {}: {e}", meta_path.display()))?;
        validate_vptree_metadata(&meta, count)?;

        let file =
            File::open(&nodes_path).map_err(|e| format!("open {}: {e}", nodes_path.display()))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("mmap {}: {e}", nodes_path.display()))?;
        let expected_len = count * VPTREE_NODE_SIZE;
        if mmap.len() != expected_len {
            return Err(format!(
                "{} size {} != expected {expected_len}",
                nodes_path.display(),
                mmap.len()
            ));
        }
        let expected_checksum = parse_checksum64(&meta.checksum64)?;
        let actual_checksum = checksum64(&mmap);
        if actual_checksum != expected_checksum {
            return Err(format!(
                "{} checksum {:016x} != metadata {:016x}",
                nodes_path.display(),
                actual_checksum,
                expected_checksum
            ));
        }

        Ok(Self {
            nodes: VpNodeStorage::Mmapped(mmap),
        })
    }

    fn write(&self, out_dir: &Path, count: usize) -> Result<(), String> {
        fs::create_dir_all(out_dir)
            .map_err(|e| format!("create_dir_all {}: {e}", out_dir.display()))?;
        let path = out_dir.join(VPTREE_NODES_FILE);
        let file = File::create(&path).map_err(|e| format!("create {}: {e}", path.display()))?;
        let mut writer = BufWriter::with_capacity(1 << 20, file);
        let mut checksum = FNV_OFFSET;
        for i in 0..count {
            let bytes = self.node(i as u32).to_bytes();
            checksum = checksum64_update(checksum, &bytes);
            writer
                .write_all(&bytes)
                .map_err(|e| format!("write {}: {e}", path.display()))?;
        }
        writer
            .flush()
            .map_err(|e| format!("flush {}: {e}", path.display()))?;

        let meta = serde_json::json!({
            "count": count,
            "node_count": count,
            "dims": DIMS,
            "node_size": VPTREE_NODE_SIZE,
            "root": 0,
            "distance": "squared_euclidean_i16",
            "tie_break": "distance_original_index",
            "checksum64": format!("{checksum:016x}"),
        });
        let mut s = serde_json::to_string_pretty(&meta).map_err(|e| format!("encode meta: {e}"))?;
        s.push('\n');
        let meta_path = out_dir.join(VPTREE_METADATA_FILE);
        fs::write(&meta_path, s).map_err(|e| format!("write {}: {e}", meta_path.display()))?;
        Ok(())
    }

    fn search(&self, refs: &[i16], labels: &[u8], q: &[i16; DIMS]) -> [Neighbor; K] {
        let mut top = TopK::new();
        self.search_node(0, refs, labels, q, &mut top);
        top.into_sorted()
    }

    fn search_node(
        &self,
        node_idx: u32,
        refs: &[i16],
        labels: &[u8],
        q: &[i16; DIMS],
        top: &mut TopK,
    ) {
        if node_idx == NONE {
            return;
        }
        let node = self.node(node_idx);
        let point = ref_at(refs, node.point_index);
        let distance = sq_dist_14(q, point);
        top.push(Neighbor {
            distance,
            index: node.point_index,
            label: labels[node.point_index as usize],
        });

        let threshold = (node.threshold_sq as f64).sqrt();
        let d = (distance as f64).sqrt();
        let tau = (top.tau_sq() as f64).sqrt();
        let may_need_left = d - tau <= threshold + f64::EPSILON;
        let may_need_right = d + tau >= threshold - f64::EPSILON;

        if d < threshold {
            if may_need_left {
                self.search_node(node.left, refs, labels, q, top);
            }
            if may_need_right {
                self.search_node(node.right, refs, labels, q, top);
            }
        } else {
            if may_need_right {
                self.search_node(node.right, refs, labels, q, top);
            }
            if may_need_left {
                self.search_node(node.left, refs, labels, q, top);
            }
        }
    }

    fn node(&self, idx: u32) -> VpNode {
        match &self.nodes {
            VpNodeStorage::Owned(nodes) => nodes[idx as usize],
            VpNodeStorage::Mmapped(m) => {
                let start = idx as usize * VPTREE_NODE_SIZE;
                VpNode::from_bytes(&m[start..start + VPTREE_NODE_SIZE])
            }
        }
    }
}

fn build_vptree_node(refs: &[i16], ids: &mut [u32], nodes: &mut Vec<VpNode>) -> u32 {
    let node_idx = nodes.len() as u32;
    let point_index = ids[0];
    nodes.push(VpNode {
        point_index,
        threshold_sq: 0,
        left: NONE,
        right: NONE,
    });
    if ids.len() == 1 {
        return node_idx;
    }

    let point = ref_at(refs, point_index);
    let mut pairs: Vec<(u32, u64)> = ids[1..]
        .iter()
        .map(|&id| (id, sq_dist_14(point, ref_at(refs, id))))
        .collect();
    let median = pairs.len() / 2;
    pairs.select_nth_unstable_by(median, |a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
    for (slot, (id, _)) in ids[1..].iter_mut().zip(pairs.iter()) {
        *slot = *id;
    }

    let threshold_sq = pairs[median].1;
    let (left_ids, right_ids) = ids[1..].split_at_mut(median);
    let left = if left_ids.is_empty() {
        NONE
    } else {
        build_vptree_node(refs, left_ids, nodes)
    };
    let right = if right_ids.is_empty() {
        NONE
    } else {
        build_vptree_node(refs, right_ids, nodes)
    };
    nodes[node_idx as usize] = VpNode {
        point_index,
        threshold_sq,
        left,
        right,
    };
    node_idx
}

fn validate_vptree_metadata(meta: &VpTreeMetadata, count: usize) -> Result<(), String> {
    if meta.count != count {
        return Err(format!("vptree count {} != refs count {count}", meta.count));
    }
    if meta.node_count != count {
        return Err(format!("vptree node_count {} != {count}", meta.node_count));
    }
    if meta.dims != DIMS {
        return Err(format!("vptree dims {} != {DIMS}", meta.dims));
    }
    if meta.node_size != VPTREE_NODE_SIZE {
        return Err(format!(
            "vptree node_size {} != {VPTREE_NODE_SIZE}",
            meta.node_size
        ));
    }
    if meta.root != 0 {
        return Err(format!("vptree root {} != 0", meta.root));
    }
    Ok(())
}

impl VpNode {
    fn to_bytes(self) -> [u8; VPTREE_NODE_SIZE] {
        let mut out = [0u8; VPTREE_NODE_SIZE];
        out[0..4].copy_from_slice(&self.point_index.to_le_bytes());
        out[4..12].copy_from_slice(&self.threshold_sq.to_le_bytes());
        out[12..16].copy_from_slice(&self.left.to_le_bytes());
        out[16..20].copy_from_slice(&self.right.to_le_bytes());
        out
    }

    fn from_bytes(b: &[u8]) -> Self {
        Self {
            point_index: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            threshold_sq: u64::from_le_bytes([b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11]]),
            left: u32::from_le_bytes([b[12], b[13], b[14], b[15]]),
            right: u32::from_le_bytes([b[16], b[17], b[18], b[19]]),
        }
    }
}

fn ref_at(refs: &[i16], idx: u32) -> &[i16] {
    let start = idx as usize * DIMS;
    &refs[start..start + DIMS]
}

fn checksum64(bytes: &[u8]) -> u64 {
    checksum64_update(FNV_OFFSET, bytes)
}

fn checksum64_update(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn parse_checksum64(s: &str) -> Result<u64, String> {
    u64::from_str_radix(s, 16).map_err(|e| format!("parse checksum64 {s}: {e}"))
}

#[inline]
fn sq_dist_14(q: &[i16], r: &[i16]) -> u64 {
    let mut acc: u64 = 0;
    for i in 0..DIMS {
        let dx = q[i] as i32 - r[i] as i32;
        let dx = dx as i64;
        acc += (dx * dx) as u64;
    }
    acc
}

#[cfg_attr(not(test), allow(dead_code))]
#[inline]
fn arg_max(a: &[u64; K]) -> usize {
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

    fn write_refs_file(dir: &Path, refs: &[i16]) {
        let mut bytes = Vec::with_capacity(refs.len() * 2);
        for v in refs {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        std::fs::write(dir.join("refs.i16.bin"), bytes).unwrap();
    }

    fn write_labels_file(dir: &Path, labels: &[u8]) {
        std::fs::write(dir.join("labels.bin"), labels).unwrap();
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
    fn sq_dist_uses_u64_for_full_range() {
        let q = [-10000i16; DIMS];
        let r = [10000i16; DIMS];
        assert_eq!(sq_dist_14(&q, &r), 5_600_000_000);
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
    fn brute_force_top5_uses_original_index_tie_break() {
        let mut refs = Vec::with_capacity(6 * DIMS);
        for _ in 0..6 {
            refs.extend_from_slice(&[0i16; DIMS]);
        }
        let labels = vec![LABEL_LEGIT; 6];
        let idx = build_index(refs, labels);
        let neighbors = idx.brute_force_top5(&zeros());
        let indices: Vec<u32> = neighbors.iter().map(|n| n.index).collect();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn vptree_top5_matches_bruteforce_with_ties() {
        let refs = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ];
        let labels = vec![
            LABEL_LEGIT,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_FRAUD,
        ];
        let idx = Index::from_parts_with_mode(
            refs.into_boxed_slice(),
            labels.into_boxed_slice(),
            SearchMode::VpTree,
        )
        .unwrap();
        assert_eq!(idx.top5(&zeros()), idx.brute_force_top5(&zeros()));
    }

    #[test]
    fn vptree_artifacts_roundtrip_match_bruteforce() {
        let refs = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
            20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
        ];
        let labels = vec![
            LABEL_LEGIT,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_FRAUD,
            LABEL_LEGIT,
            LABEL_FRAUD,
        ];
        let dir = std::env::temp_dir().join("rinha_vptree_artifacts_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_refs_file(&dir, &refs);
        write_labels_file(&dir, &labels);
        write_vptree_artifacts(&refs, &dir).unwrap();

        let disk = Index::load(&dir, SearchMode::VpTree).unwrap();
        let brute = build_index(refs, labels);
        assert_eq!(disk.top5(&zeros()), brute.brute_force_top5(&zeros()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vptree_load_missing_artifacts_fails_without_bruteforce_fallback() {
        let refs = vec![0i16; K * DIMS];
        let labels = vec![LABEL_LEGIT; K];
        let dir = std::env::temp_dir().join("rinha_vptree_missing_artifacts");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_refs_file(&dir, &refs);
        write_labels_file(&dir, &labels);

        assert!(Index::load(&dir, SearchMode::VpTree).is_err());
        assert!(Index::load(&dir, SearchMode::BruteForce).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vptree_load_corrupt_artifact_fails_without_bruteforce_fallback() {
        let refs = vec![0i16; K * DIMS];
        let labels = vec![LABEL_LEGIT; K];
        let dir = std::env::temp_dir().join("rinha_vptree_corrupt_artifact");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_refs_file(&dir, &refs);
        write_labels_file(&dir, &labels);
        write_vptree_artifacts(&refs, &dir).unwrap();

        let nodes_path = dir.join(VPTREE_NODES_FILE);
        let mut nodes = std::fs::read(&nodes_path).unwrap();
        nodes[0] ^= 1;
        std::fs::write(&nodes_path, nodes).unwrap();

        assert!(Index::load(&dir, SearchMode::VpTree).is_err());
        assert!(Index::load(&dir, SearchMode::BruteForce).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
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
