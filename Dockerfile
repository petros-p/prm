# ---- Stage 1: "builder" ----
# This image has the full Rust toolchain (rustc, cargo) on top of a
# minimal Debian base ("slim"). It only exists to compile the binary —
# it is never shipped anywhere; the final image below throws it away.
FROM rust:1.93.1-slim-bookworm AS builder

WORKDIR /app

# Copy just the dependency manifests first, not the source code yet.
# Docker builds an image instruction-by-instruction, caching the
# result of each instruction as a "layer". If the *inputs* to a layer
# haven't changed since the last build, Docker reuses the cached
# result instead of re-running it. By copying Cargo.toml/Cargo.lock
# before the source, editing a .rs file later won't invalidate this
# layer — Cargo won't have to re-fetch/re-compile every dependency
# from scratch on every rebuild, only the final `cargo build` layer.
COPY Cargo.toml Cargo.lock ./

# Now copy the actual source and compile.
# Cargo.toml's `default` feature set includes `ai` (so a plain `cargo
# run` on a dev machine gets ai-log/voice-log/inbox), but this image
# explicitly opts back out with --no-default-features: whisper-rs
# compiles C++ code via a build script, which needs cmake/a C++
# compiler that this slim builder deliberately doesn't have, just to
# run `add-person`/`log`/`stats`. The ai-gated commands also depend on
# a locally-running Ollama and, for voice-log, a microphone — neither
# of which make sense inside a container anyway.
COPY src ./src
RUN cargo build --release --no-default-features

# ---- Stage 2: runtime ----
# A fresh, minimal image with no Rust toolchain, no build cache, no
# source code — just enough OS to run one compiled binary. This is
# the entire point of a *multi-stage* build: instead of shipping the
# ~1GB+ builder image (compilers, intermediate object files, the
# downloaded crate registry), the final image only contains what
# `COPY --from=builder` explicitly pulls out of it.
FROM debian:bookworm-slim

WORKDIR /app

# Pull *only* the compiled binary out of the builder stage, by path.
# Nothing else from that stage makes it in here.
COPY --from=builder /app/target/release/prm /usr/local/bin/prm

# PRM stores its SQLite database at .data/prm.db, relative to the
# working directory. Declaring it a VOLUME is a hint that this path
# holds state that should live outside the container's own writable
# layer. In practice, we'll pass `-v <host folder>:/app/.data` on
# `docker run`, which *mounts* a real folder from your machine at that
# path inside the container — so `people`/`log` entries you create end
# up in a file on your own disk, and survive `docker rm`-ing the
# container (containers are meant to be disposable; your data isn't).
VOLUME /app/.data

ENTRYPOINT ["prm", "--file", ".data/prm.db"]
