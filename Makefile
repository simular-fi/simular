build:
	poetry run maturin develop

test: build
	poetry run pytest tests/*

rust_tests: 
	cargo test --no-default-features
