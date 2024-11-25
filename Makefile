PINNED_TOOLCHAIN := $(shell cat contracts/rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rust-src --toolchain ${PINNED_TOOLCHAIN}

.PHONY:	build-contract
build-contract:
	cd contracts && RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -p cowl-cep18
	wasm-strip target/wasm32-unknown-unknown/release/cowl_cep18.wasm

.PHONY:	build-all-contracts
build-all-contracts: build-contract
	cd contracts && RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -p cep18-test-contract
	wasm-strip target/wasm32-unknown-unknown/release/cep18_test_contract.wasm

setup-test: build-all-contracts
	mkdir -p tests/wasm
	cp ./target/wasm32-unknown-unknown/release/cowl_cep18.wasm tests/wasm
	cp ./target/wasm32-unknown-unknown/release/cep18_test_contract.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
	cd contracts && cargo clippy --bins --target wasm32-unknown-unknown -Z build-std=std,panic_abort -- -D warnings
	cd contracts && cargo clippy --lib --target wasm32-unknown-unknown -Z build-std=std,panic_abort -- -D warnings
	cd contracts && cargo clippy --lib --target wasm32-unknown-unknown -Z build-std=std,panic_abort --no-default-features -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contracts && cargo fmt -- --check
	cd tests && cargo +$(PINNED_TOOLCHAIN) fmt -- --check

format:
	cd contracts && cargo fmt
	cd tests && cargo +$(PINNED_TOOLCHAIN) fmt

clean:
	cd contracts && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm