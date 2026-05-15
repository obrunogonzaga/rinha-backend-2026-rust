# Bake reference artifacts into the runtime image

Under the Rinha 2026 budget of 1 CPU / 350 MB total across all services with two
API replicas, each API container needs the 84 MB `data/refs.i16.bin` plus the
3 MB `data/labels.bin`. We considered baking the artifacts into the image (each
container loads its own mmap) versus bind-mounting a shared volume so the
kernel page cache can dedup across containers (open question recorded in
`memory/plan.md` before Slice 4).

We decided to **bake the artifacts into the runtime image**.

Rationale: per-API RSS budget of 170 MB has ~50 MB of headroom over the
observed Slice 3 darwin/arm64 baseline (~100 MB peak under cgroup), enough that
explicit page sharing is not load-bearing for the budget. The shared-volume
alternative would require either (a) versioning artifacts in `submission/`
(forbidden by the "no source code on submission" rule, and the file is too big
to live there anyway) or (b) an init service in compose that copies the data
out of the API image into a named volume — extra orchestration for an
optimization we can quantify after the first official measurement.

If the official Engine measurement shows per-container RSS hitting the limit,
revisit by introducing a shared-volume topology in a follow-up slice (Slice 5+).
This ADR is then superseded.
