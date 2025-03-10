.PHONY: build
build:
	cargo build --no-default-features

.PHONY: build-op
build-op:
	cargo build --features optimism

.PHONE: release
release:
	cargo build --release

.PHONY: test
test:
	cargo test

.PHONY: test-op
test-op:
	cargo test --features optimism

.PHONY: clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: fmt-check
fmt-check:
	cargo fmt --all --check

.PHONY: clippy
clippy:
	cargo clippy --all --all-features -- -D warnings

.PHONY: taplo
taplo:
	taplo format

.PHONY: taplo-check
taplo-check:
	taplo format --check

.PHONY: deny-check
deny-check:
	cargo deny --all-features check

.PHONY: pre-release
pre-release:
	make fmt
	make clippy
	make test
	make test-op
	make taplo-check
	make deny-check