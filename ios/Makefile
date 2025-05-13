MAKEFILE_DIR := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
OUTPUT_DIR ?= $(MAKEFILE_DIR)

RUSTUP ?= rustup
CARGO ?= cargo

CARGO_MANIFEST_DIR ?= $(MAKEFILE_DIR)/../rust
CARGO_MANIFEST_PATH := $(CARGO_MANIFEST_DIR)/Cargo.toml

CARGO_TARGET_AARCH64 := aarch64-apple-ios
CARGO_TARGET_AARCH64_SIM := aarch64-apple-ios-sim
CARGO_TARGET_X86_64 := x86_64-apple-ios
CARGO_TARGET_UNIVERSAL_SIM := universal-apple-ios-sim

CARGO_TARGET_DIR ?= $(CARGO_MANIFEST_DIR)/target
cargo_target_dir = $(CARGO_TARGET_DIR)/$(1)/release
cargo_target_path = $(call cargo_target_dir,$(1))/libacubench.a

HEADERS_DIR := $(CARGO_MANIFEST_DIR)/ffi/include

XCFRAMEWORK_TARGET_AARCH64 = ios-arm64
XCFRAMEWORK_TARGET_SIM = ios-arm64_x86_64-simulator

XCFRAMEWORK_PATH := $(OUTPUT_DIR)/AcubenchFFI.xcframework
xcframework_headers_dir = $(XCFRAMEWORK_PATH)/$(1)/Headers
xcframework_modulemap_path = $(call xcframework_headers_dir,$(1))/module.modulemap

cargo-prepare:
	@echo "Preparing..."
	$(RUSTUP) target add $(CARGO_TARGET_AARCH64)
	$(RUSTUP) target add $(CARGO_TARGET_X86_64)

cargo-build: cargo-prepare
	@echo "Building..."
	$(CARGO) build --release --target $(CARGO_TARGET_AARCH64) --manifest-path $(CARGO_MANIFEST_PATH) --target-dir $(CARGO_TARGET_DIR)
	$(CARGO) build --release --target $(CARGO_TARGET_AARCH64_SIM) --manifest-path $(CARGO_MANIFEST_PATH) --target-dir $(CARGO_TARGET_DIR)
	$(CARGO) build --release --target $(CARGO_TARGET_X86_64) --manifest-path $(CARGO_MANIFEST_PATH) --target-dir $(CARGO_TARGET_DIR)

create-xcframework: cargo-build
	@echo "Packing..."
	@mkdir -p $(call cargo_target_dir,$(CARGO_TARGET_UNIVERSAL_SIM))
	lipo -create \
		$(call cargo_target_path,$(CARGO_TARGET_AARCH64_SIM)) \
		$(call cargo_target_path,$(CARGO_TARGET_X86_64)) \
		-output $(call cargo_target_path,$(CARGO_TARGET_UNIVERSAL_SIM))


	rm -rf $(XCFRAMEWORK_PATH)
	xcodebuild -create-xcframework \
		-library $(call cargo_target_path,$(CARGO_TARGET_AARCH64)) \
		-headers $(HEADERS_DIR) \
		-library $(call cargo_target_path,$(CARGO_TARGET_UNIVERSAL_SIM)) \
		-headers $(HEADERS_DIR) \
		-output $(XCFRAMEWORK_PATH)

generate-modulemap: create-xcframework
	@echo "Generate Module Map..."
	@for path in $(call xcframework_modulemap_path,$(XCFRAMEWORK_TARGET_AARCH64)) $(call xcframework_modulemap_path,$(XCFRAMEWORK_TARGET_SIM)); do \
		echo "module AcubenchFFI {" > $$path; \
		echo "    header \"ffi.h\"" >> $$path; \
		echo "    export *" >> $$path; \
		echo "}" >> $$path; \
		echo "module.modulemap successfully written out to: $$path"; \
	done

build: generate-modulemap