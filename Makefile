prepare:
	rustup target add wasm32-unknown-unknown

.PHONY:	build-contract
build-contract:
	cd cep18 && cargo build --release --target wasm32-unknown-unknown
	cd cep18-test-contract && cargo build --release --target wasm32-unknown-unknown
	wasm-strip ./cep18/target/wasm32-unknown-unknown/release/cep18.wasm
	wasm-strip ./cep18-test-contract/target/wasm32-unknown-unknown/release/cep18_test_contract.wasm

setup-test: build-contract
	mkdir -p tests/wasm
	cp ./cep18/target/wasm32-unknown-unknown/release/cep18.wasm tests/wasm
	cp ./cep18-test-contract/target/wasm32-unknown-unknown/release/cep18_test_contract.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
	cd cep18 && cargo clippy --all-targets -- -D warnings
	cd cep18-test-contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

format:
	cd cep18 && cargo fmt
	cd cep18-test-contract && cargo fmt
	cd tests && cargo fmt

check-lint: clippy
	cd cep18 && cargo fmt -- --check
	cd cep18-test-contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd cep18 && cargo fmt
	cd cep18-test-contract && cargo fmt
	cd tests && cargo fmt

clean:
	cd cep18 && cargo clean
	cd cep18-test-contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
