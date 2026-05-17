# Bake the search index into the runtime image

Slice 5 needs to reduce the cost of exact kNN without changing classifier
output. We decided to generate the structured search index at Docker build
time and bake it into the runtime image, alongside the existing reference
artifacts. The dataset is fixed during evaluation, so build-time indexing keeps
startup deterministic, avoids healthcheck risk, and lets every published image
carry a checksumed index for the exact code/data pair it serves.

## Exactness of VP-Tree pruning

The classifier must return the same `approved`/`fraud_score` as brute force
for *every* query the Engine sends, not only the queries in our local
fixtures. VP-Tree pruning is therefore evaluated in exact integer space: the
triangle-inequality bounds `d <= threshold + tau` and `threshold <= d + tau`
are tested via `sqrt_le_sum`, which decides `sqrt(a2) <= sqrt(b2) + sqrt(c2)`
using only `i128`/`u128` arithmetic on the stored squared distances. No
`f64::sqrt` is used on the search path, so there is no rounding window in
which a true neighbor's subtree could be wrongly pruned. The bound is
provably the true Euclidean bound for the full `i16`×14 range (max squared
distance 5.6e9, products bounded well inside `u128`).

The earlier float formulation padded with `f64::EPSILON`, which is ~5 orders
of magnitude smaller than the sqrt rounding error at these magnitudes and did
not robustly guarantee conservative pruning. Integer pruning supersedes it.

## Validation

Equivalence is checked by the `validate_equivalence` binary in three layers:
all `test/test-data.json` entries against the official expected
labels/scores; a strided top-5 sample against brute force; and a
deterministic differential fuzz of reference-derived queries (exact,
near-perturbed, far-perturbed) plus uniform-random points. The PRNG is a
fixed-seed SplitMix64 so the sample reproduces bit-for-bit across machines.
Confirmed locally against the full 3,000,000-reference dataset (5000 random
queries, 0 mismatches).

## Behavior change: `count < K`

`Index::load` now rejects a reference set smaller than `K` instead of
silently scoring against `i64::MAX`-distance placeholder neighbors. The
evaluation dataset has millions of references, so this only affects
degenerate/test inputs; failing loud is the desired contract.
