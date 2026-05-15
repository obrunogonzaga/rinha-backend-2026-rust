# Compile the runtime binary with `target-cpu=x86-64-v3`

The brute-force scan in `src/index.rs` (`sq_dist_14`) is a scalar Rust loop
that relies on LLVM auto-vectorization. The Rust default for
`x86_64-unknown-linux-gnu` is the "x86-64" baseline (SSE2 only, 128-bit), so
auto-vec emits 128-bit code processing 8× `i16` per op. The official test
hardware is a Mac Mini Late 2014 (i5-4278U, Haswell) which supports AVX2,
FMA3, and BMI1/BMI2 — and so does every realistic linux/amd64 cloud host
since ~2015.

We set `target-cpu=x86-64-v3` in `.cargo/config.toml`, scoped to the
`x86_64-unknown-linux-gnu` target. This enables AVX2 (256-bit, 16× `i16` per
op) and FMA without naming a specific microarchitecture. Native macOS arm64
builds (used for `cargo test` / dev iteration) are unaffected.

Trade-off: the resulting image will not run on pre-Haswell CPUs (Sandy Bridge,
Ivy Bridge). Anyone trying to run our `submission` compose on such hardware
gets `SIGILL` at startup. Acceptable because (1) the official environment is
Haswell, (2) the public x86-64-v3 microarchitecture level is the established
floor for modern Linux distributions (RHEL 10, glibc-hwcaps).

If a future evaluator runs the compose on an older CPU and reports the
`SIGILL`, downgrade to `x86-64-v2` (Nehalem / SSE4.2) and re-publish under a
new tag.
