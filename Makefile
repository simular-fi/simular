build:
	poetry run maturin develop

release:
	poetry run maturin develop --release

test: build
	poetry run pytest tests/*

rust_tests:
	cargo test --no-default-features

benchit:
	poetry run python bench/simple.py

docs: 
	mdbook build docs/
