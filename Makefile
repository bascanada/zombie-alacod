PROFILE ?= dev


CURRENT_TAG := $(shell git describe --tags --exact-match HEAD 2>/dev/null)

ifeq ($(CURRENT_TAG),)
	LATEST_TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    SHORT_SHA := $(shell git rev-parse --short HEAD)
	VERSION := $(LATEST_TAG)-$(SHORT_SHA)
else
	VERSION := $(CURRENT_TAG)
endif


ifeq ($(PROFILE), dev)
	export MODE_DIR := debug
	export CARGO_TARGET_DIR := ./target
endif

ifeq ($(PROFILE), prod)
	export MODE_DIR := release
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

dep_hosting:
	npm install -g live-server

dep_matchbox:
	cargo install matchbox_server

dep: dep_web dep_hosting dep_matchbox

# Dev run

map_preview:
	cargo run --example map_preview $(ARGS) --features native

map_generation:
	cargo run --example map_generation $(ARGS)

map_explorer:
	cargo run --example map_explorer $(ARGS) --features native


character_tester:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- --local-port 7000 --players localhost

character_tester_matchbox:
	APP_VERSION=$(VERSION) cargo run --example character_tester $(ARGS) --features native -- --number-player 2 --matchbox "wss://matchbox.bascanada.org" --lobby test_2 --players localhost remote

matchbox_server:
	APP_VERSION=$(VERSION) cargo run -p matchbox_server

host_website:
	cd website && APP_VERSION=$(VERSION) npm run dev

cp_asset:
	cp -r ./assets ./website/static/

build_docker_matchbox_server:
	docker build -f ./crates/matchbox_server/Dockerfile ./crates/matchbox_server/ -t ghcr.io/bascanada/matchbox_server:latest

build_map_preview_web:
	APP_VERSION=$(VERSION) cargo build --example map_preview --target wasm32-unknown-unknown --features bevy_ecs_tilemap/atlas $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/map_preview --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/map_preview.wasm

build_character_tester_web:
	APP_VERSION=$(VERSION) cargo build --example character_tester --target wasm32-unknown-unknown $(RELEASE)
	wasm-bindgen --out-dir ./website/static/$(VERSION)/character_tester --out-name wasm --target web $(CARGO_TARGET_DIR)/wasm32-unknown-unknown/$(MODE_DIR)/examples/character_tester.wasm

build_website: cp_asset build_map_preview_web build_character_tester_web
	cd website && APP_VERSION=$(VERSION) npm run build	

build_docker_website: build_website
	docker build --build-arg APP_VERSION=$(VERSION) -f ./website/Dockerfile ./website -t ghcr.io/bascanada/zombie-alacod:latest

# Publish
push_docker_matchbox_server:
	docker push ghcr.io/bascanada/matchbox_server:latest

push_docker_website:
	docker push ghcr.io/bascanada/zombie-alacod:latest


print_version:
	@echo "Current Tag: $(CURRENT_TAG)"
	@echo "Version: $(VERSION)"

