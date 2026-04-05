USERNAME = rockcen
PROJECT = handy-rat
BUILD_DIR = wasm_builds
DIST_DIR = target/bevy_web/web-release
DIST_FOLDER = bevy_game
CHANNEL = html5
#last tag name, fallback to v1.0.0 if no tags exist
VERSION := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v1.0.0")
ZIP_FILE := $(BUILD_DIR)/$(PROJECT)-$(VERSION).zip

colon = :

build-wasm:
	bevy build --release --yes --features web,backend web --bundle

upload: build-wasm archive
	butler push $(ZIP_FILE) $(USERNAME)/$(PROJECT)$(colon)$(CHANNEL) --userversion $(VERSION)

archive: $(DIST_DIR)/$(DIST_FOLDER)
	mkdir -p $(BUILD_DIR)
	cd $(DIST_DIR) && zip -r $(CURDIR)/$(ZIP_FILE) $(DIST_FOLDER)

$(DIST_DIR)/$(DIST_FOLDER):
	@echo "Dist folder does not exist"
