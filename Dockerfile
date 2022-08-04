FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /meta
COPY . /meta
RUN cargo build --locked --release

FROM debian:stretch-slim

WORKDIR /meta

COPY --from=builder /meta/target/release/meta-node ./bin/meta-node

EXPOSE 30333 9944
ENTRYPOINT ["./bin/meta-node"]