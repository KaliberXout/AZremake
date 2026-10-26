# syntax=docker/dockerfile:1
#
# Builds Azeroth Survivors without needing Rust/cargo on the host: the wasm client
# (scripts/web-setup.sh + scripts/web-build.sh) and the native wenilla-host binary that serves
# it, then ships just the compiled output in a small runtime image.
#
# The game data (a WoW 1.12.1 client's Data/*.MPQ files) is NOT baked into the image — this repo
# ships no Blizzard content. Mount it at /data/Data at runtime (see docker-compose.yaml); that
# nested `Data/` is where scripts/fetch-data.sh always extracts the MPQs, matching the layout
# `benilla_formats::Chain::open` expects.

FROM rust:1.98.1-bookworm AS builder

# curl/unzip: scripts/web-setup.sh fetches wasi-sdk + binaryen.
# build-essential/pkg-config: compiling the vendored Lua VM to wasm and building wenilla-host.
# brotli: optional precompression in scripts/web-build.sh (skipped silently if absent).
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates unzip brotli build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

RUN scripts/web-setup.sh
RUN scripts/web-build.sh
RUN cargo build --release -p wenilla-host

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /src/target/release/wenilla-host /app/wenilla-host
COPY --from=builder /src/web/dist /app/web/dist

ENV RUST_LOG=info
EXPOSE 8090

ENTRYPOINT ["/app/wenilla-host"]
CMD ["--bind", "0.0.0.0:8090", "--www", "/app/web/dist", "--data", "/data/Data"]
