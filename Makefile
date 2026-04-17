# bcinr Makefile (Delegates to cargo-make)

.PHONY: all check build test test-lib bench bench-report clippy fmt clean

all:
	cargo make all

check:
	cargo make check

build:
	cargo make build

test:
	cargo make test

test-lib:
	cargo make test-lib

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
