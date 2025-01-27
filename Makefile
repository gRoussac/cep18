PINNED_TOOLCHAIN := $(shell cat contracts/rust-toolchain)

prepare:
	rustup install ${PINNED_TOOLCHAIN}
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rust-src --toolchain ${PINNED_TOOLCHAIN}

.PHONY:	build-contract
build-contract:
	RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -p cep18
	wasm-strip target/wasm32-unknown-unknown/release/cep18.wasm

.PHONY:	build-all-contracts
build-all-contracts: build-contract
	RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -p cep18-test-contract
	wasm-strip target/wasm32-unknown-unknown/release/cep18_test_contract.wasm

.PHONY:	setup-test
setup-test: build-all-contracts copy-wasm

.PHONY:	copy-wasm
copy-wasm:
	mkdir -p tests/wasm
	cp ./target/wasm32-unknown-unknown/release/cep18.wasm tests/wasm
	cp ./target/wasm32-unknown-unknown/release/cep18_test_contract.wasm tests/wasm

native-test: setup-test
	cd tests && cargo test --lib should_transfer_account_to_account

test: setup-test
	cd tests && cargo test --lib

clippy:
	cd contracts clippy --all-targets -- -D warnings
	cd contracts/test-contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

format:
	cd contracts && cargo fmt
	cd contracts/test-contract && cargo fmt
	cd tests && cargo fmt

check-lint: clippy
	cd contracts && cargo fmt -- --check
	cd contracts/test-contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy format

clean:
	cd contracts && cargo clean
	cd contracts/test-contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
