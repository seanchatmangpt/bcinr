# bcinr Makefile (Delegates to cargo-make)

.PHONY: check build test bench bench-report clippy fmt clean

check:
	cargo make check

build:
	cargo make build

test:
	cargo make test

bench:
	cargo make bench

bench-report:
	cargo make bench-report

clippy:
	cargo make clippy

fmt:
	cargo make fmt

clean:
	cargo make clean
