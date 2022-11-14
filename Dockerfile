FROM ubuntu:latest

WORKDIR /metachain
#
# Requires copy the binary to `build` folder beforehand
COPY target/* /metachain
#
# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944
#
#VOLUME ["/data"]
#
## ENTRYPOINT ["/metachain/meta-node"]

# Update system
RUN apt-get update

# Install curl
RUN apt-get install curl -y

# Install Nodejs
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | /bin/sh - && \
    apt-get install nodejs

# Copy all files to container
COPY . /metachain

WORKDIR /metachain/ts-tests

# Install test dependencies
RUN npm install -g npm@9.1.1
RUN npm i



