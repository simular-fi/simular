build:
	poetry run maturin develop

release:
	poetry run maturin develop --release

test: build
	poetry run pytest tests/*

rust_tests:
	cargo test --no-default-features
