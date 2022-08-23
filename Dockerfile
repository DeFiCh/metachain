FROM rust:buster as builder

RUN apt-get update && \
    apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev llvm
RUN rustup toolchain install nightly-2022-07-24
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2022-07-24

WORKDIR /metachain
COPY . /metachain

RUN cargo build --release --all

# ===== SECOND STAGE ======
FROM phusion/baseimage:focal-1.2.0

RUN useradd -m -u 1000 -U -s /bin/sh -d /metachain metachain

COPY --from=builder /metachain/target/release/meta-node /usr/local/bin

RUN chmod +x /usr/local/bin/meta-node

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/meta-node"]