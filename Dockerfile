FROM ubuntu:22.04 AS builder

RUN apt update && apt upgrade -y
RUN apt install -y curl

WORKDIR /metachain
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain none --profile minimal -y

ENV PATH=$PATH:/root/.cargo/bin
RUN apt install -y cmake clang

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
# TODO: docker layer cache, blocked by https://github.com/rust-lang/cargo/issues/2644
# RUN cargo fetch

COPY meta ./meta
ARG PROFILE=release
ENV PROFILE ${PROFILE}
RUN cargo build --$PROFILE --all


FROM ubuntu:22.04 AS runner
WORKDIR /metachain
COPY --from=builder /metachain/target/release/meta-node .

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9333 9944 9615 39333 19933 19944  # from https://github.com/DeFiCh/metachain/blob/d3f4a9b36eb25d7340a8b138795882cada7c60e5/packages/network/src/NetworkConfig.ts

VOLUME ["/data"]
ENTRYPOINT ["/metachain/meta-node"]
