# Minimal multi-stage build for the `castle` CLI binary -- follows the
# same convention platform-console's own services/*/Dockerfile use
# (single COPY of prebuilt/source artifacts, non-root runtime user, no
# shell needed at runtime since the container's command is the compiled
# binary invoked directly, never `sh -c`).
#
# Builder: Cargo.toml declares rust-version = "1.82" as a floor, but
# transitive deps pulled in via clap-noun-verb (idna_adapter needing
# cargo's edition2024 support, then icu_* crates needing rustc>=1.88)
# push the real minimum higher -- confirmed live by two failed builds.
# 1.97 matches this host's own installed rustc (`rustc --version`), the
# toolchain the crate is actually developed against.
FROM rust:1.97-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY configs ./configs
RUN cargo build --release --bin castle

# Runtime: debian-slim, not distroless -- castle's own CLAUDE.md documents
# a compiler-enforced sealed `admit_construct_for_do` gate, not a runtime
# sandboxing requirement, so a small, patchable base is preferred here
# over distroless's slightly smaller but harder-to-patch image. Runs as a
# real non-root user, never root.
FROM debian:bookworm-slim
RUN groupadd -r castle && useradd -r -g castle -u 10001 castle
COPY --from=builder /build/target/release/castle /usr/local/bin/castle
USER castle:castle
ENTRYPOINT ["/usr/local/bin/castle"]
