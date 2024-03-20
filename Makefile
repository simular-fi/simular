build:
	hatch run dev:maturin develop

release:
	hatch run dev:maturin develop --release

test: build
	hatch run dev:pytest tests/*

rust_tests:
	cargo test --no-default-features

benchit:
	hatch run dev:python bench/simple.py

docs: 
	mdbook build docs/

make shell:
	hatch --env dev shell
