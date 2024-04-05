
build:
	hatch run dev:maturin develop

# this build the production release
dev_release:
	hatch run dev:maturin develop --release

test: build
	hatch run dev:pytest tests/*

rust_tests:
	cargo test --no-default-features

benchit:
	hatch run dev:python bench/simple.py

docs: 
	hatch run:dev docs

shell:
	hatch --env dev shell
