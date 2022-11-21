# Note: This is currently designed to simplify development
# To get a smaller docker image, there should be 2 images generated, in 2 stages.

FROM rustlang/rust:nightly
ARG PROFILE=release
# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev

WORKDIR /frontier
COPY . .

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

ENV PROFILE ${PROFILE}
RUN cargo build --$PROFILE --package meta-node
