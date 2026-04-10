.PHONY: build build-release fix patch profile release run minor
include make/web.mk

# minor or release major, minor, patch,
patch:
	git pull
	cargo release patch --no-publish --execute

minor:
	git pull
	cargo release minor --no-publish --execute



profile:
	cargo bloat -n 100000 --message-format json > out.json

fix:
	cargo fix --workspace --message-format=json


