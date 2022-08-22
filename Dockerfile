FROM ubuntu:22.04

WORKDIR /metachain 

# Requires copy the binary to `build` folder beforehand
COPY build/* /metachain 

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 39333 19933 19944

VOLUME ["/data"]

ENTRYPOINT ["/metachain/meta-node"]
