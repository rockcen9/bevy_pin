.PHONY: build build-release fix patch profile release run minor
include make/web.mk

# minor or release major, minor, patch,
patch:
	cargo release patch --no-publish --execute

minor:
	cargo release minor --no-publish --execute



profile:
	cargo bloat -n 100000 --message-format json > out.json

fix:
	cargo fix --workspace --message-format=json


