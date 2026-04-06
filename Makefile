.PHONY: build build-release fix patch profile release run
include make/web.mk

# minor or release major, minor, patch,
patch:
	cargo release patch --no-publish --execute

profile:
	cargo bloat -n 100000 --message-format json > out.json

fix:
	cargo fix --workspace --message-format=json


