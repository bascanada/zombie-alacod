PROFILE ?= dev

ifeq ($(PROFILE), dev)
	export MODE_DIR := debug
	export CARGO_TARGET_DIR := ./target
endif

ifeq ($(PROFILE), prod)
	export RELEASE := --release

endif

ifeq ($(TARGET), native)
	export CARGO_TARGET_DIR := ./target
endif


ifeq ($(TARGET), web)
	export RUSTFLAGS := --cfg=web_sys_unstable_apis
	export CARGO_TARGET_DIR := ./target_wasm
endif




# ALL

all: test format


# Misc

clean:
	@echo "Cleaning the project..."
	cargo clean

format:
	@echo "Running fmy..."
	cargo fmt --all -- --emit=files


# Test

test:
	@echo "Running tests with profile"
	cargo test


# Env



# Dependencies

dep_web:
	rustup target add wasm32-unknown-unknown
	cargo install wasm-bindgen-cli


# Dev run

map_preview:
	cargo run --example map_preview $(ARGS)

map_generation:
	cargo run --example map_generation $(ARGS)

map_explorer:
	cargo run --example map_explorer $(ARGS)

character_tester:
	cargo run --example character_tester $(ARGS)

host_website: build_website
	cd website && python3 -m http.server


# Build

build_website:
	cargo build --example map_preview --target wasm32-unknown-unknown --features bevy_ecs_tilemap/atlas $(RELEASE)
	wasm-bindgen --out-dir ./website/ --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/map_preview.wasm
	cp -r ./assets ./website/

