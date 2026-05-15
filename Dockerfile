# Slice 4 — multi-stage build for the Rinha 2026 submission.
#
# Decisions:
# - ADR-0001 (docs/adr/0001-bake-data-in-image.md): data/*.bin baked into the
#   runtime image, no shared volume.
# - ADR-0002 (docs/adr/0002-target-cpu-x86-64-v3.md): linux-gnu amd64 builds
#   are compiled with target-cpu=x86-64-v3 via .cargo/config.toml.
# - cargo-chef caches the dependency build so iteration on src/ only
#   recompiles our crate (~30 s under QEMU vs minutes from scratch).

ARG RUST_VERSION=1.88
ARG DISTROLESS_TAG=nonroot

# --- stage: chef — rust toolchain + cargo-chef ---
FROM rust:${RUST_VERSION}-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# --- stage: planner — capture dependency graph as recipe.json ---
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
RUN cargo chef prepare --recipe-path recipe.json

# --- stage: builder — cook deps, then build the crate (release+LTO) ---
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
RUN cargo build --release --locked --bin rinha_backend_2026 --bin preprocess

# --- stage: preprocessor — bake reference artifacts ---
FROM builder AS preprocessor
COPY resources/references.json.gz resources/references.json.gz
RUN mkdir -p /out/data \
    && ./target/release/preprocess resources/references.json.gz /out/data

# --- stage: runtime — distroless/cc, nonroot ---
FROM gcr.io/distroless/cc-debian12:${DISTROLESS_TAG}
LABEL org.opencontainers.image.source=https://github.com/obrunogonzaga/rinha-backend-2026-rust
LABEL org.opencontainers.image.licenses=MIT
COPY --from=builder /app/target/release/rinha_backend_2026 /app/api
COPY --from=preprocessor /out/data /data
ENV REFS_DATA_DIR=/data
EXPOSE 9999
HEALTHCHECK --interval=2s --timeout=1s --start-period=20s --retries=5 \
    CMD ["/app/api", "--healthcheck"]
ENTRYPOINT ["/app/api"]
