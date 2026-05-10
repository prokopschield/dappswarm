# dappswarm — demo helpers.
#
# Required env (set once before invoking targets):
#   STAMP_BATCH_ID   — funded postage batch id (32-byte hex)
#   DAPPSWARM_KEY    — publisher signing key (32-byte hex secp256k1 secret)
#
# Optional:
#   DAPPSWARM_GATEWAY — antd base URL (default http://127.0.0.1:1633)
#   DAPPSWARM_OWNER   — derived from DAPPSWARM_KEY at publish time;
#                       resolve/install need it explicitly.

PACKAGE := hello.dnp.dappnode.eth
BUNDLE  := fixtures/hello-dnp
INSTALL_DIR := /tmp/dappswarm-install

DEMO_PACKAGE := dappswarm-demo
DEMO_BUNDLE  := demo/dist
DEMO_INSTALL_DIR := /tmp/dappswarm-demo-install

.PHONY: doctor publish info resolve install clean-install
.PHONY: demo-build demo-publish demo-install demo-clean

doctor:
	cargo run --quiet -- doctor

publish:
	cargo run --quiet -- publish $(BUNDLE)

info:
	@if [ -z "$(DAPPSWARM_OWNER)" ]; then echo "set DAPPSWARM_OWNER=0x…"; exit 1; fi
	cargo run --quiet -- info $(PACKAGE) --owner $(DAPPSWARM_OWNER)

resolve:
	@if [ -z "$(DAPPSWARM_OWNER)" ]; then echo "set DAPPSWARM_OWNER=0x…"; exit 1; fi
	cargo run --quiet -- resolve $(PACKAGE) --owner $(DAPPSWARM_OWNER) --out $(INSTALL_DIR)

install:
	@if [ -z "$(DAPPSWARM_OWNER)" ]; then echo "set DAPPSWARM_OWNER=0x…"; exit 1; fi
	cargo run --quiet -- install $(PACKAGE) --owner $(DAPPSWARM_OWNER) --data-dir $(INSTALL_DIR)

clean-install:
	-docker compose -f $(INSTALL_DIR)/docker-compose.yml down 2>/dev/null
	-rm -rf $(INSTALL_DIR)

demo-build:
	cd demo && ./scripts/build-bundle.sh

demo-publish: demo-build
	cargo run --quiet -- publish $(DEMO_BUNDLE)

demo-install:
	@if [ -z "$(DAPPSWARM_OWNER)" ]; then echo "set DAPPSWARM_OWNER=0x…"; exit 1; fi
	cargo run --quiet -- install $(DEMO_PACKAGE) --owner $(DAPPSWARM_OWNER) --data-dir $(DEMO_INSTALL_DIR)

demo-clean:
	-docker compose -f $(DEMO_INSTALL_DIR)/docker-compose.yml down 2>/dev/null
	-rm -rf $(DEMO_INSTALL_DIR)
	-rm -rf demo/dist demo/.svelte-kit
