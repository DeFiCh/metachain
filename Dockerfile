FROM ubuntu:22.04

WORKDIR /meta 

# Requires copy the binary to `build` folder beforehand
COPY build/* /meta 

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 39333 19933 19944

VOLUME ["/data"]

ENTRYPOINT ["/metachain/meta-node"]
