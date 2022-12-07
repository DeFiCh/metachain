CARGO ?= cargo
TARGET ?=

build-native-pkg:
	CRATE_CC_NO_DEFAULTS=1 $(CARGO) build --package meta-node --release $(if $(TARGET),--target $(TARGET),)
	mkdir -p pkg/metachain/include pkg/metachain/lib
	cp target/$(if $(TARGET),$(TARGET)/,)release/libmeta_node.a pkg/metachain/lib/libmetachain.a
	cp target/libmc.hpp pkg/metachain/include/
	cp target/libmc.cpp pkg/metachain/
