.PHONY: docs

build:
	hatch run dev:maturin develop

# this builds the production release
prod_release:
	hatch run dev:maturin build

test: build
	hatch run dev:pytest tests/*

rust_tests:
	cargo test --no-default-features

benchit: prod_release
	hatch run dev:python bench/simple.py

docs: 
	hatch run dev:docs

shell:
	hatch --env dev shell
