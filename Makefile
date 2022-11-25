prog = kato-rs

test:
	cargo test -- --test-threads=1

build:
	cargo build $(release)

build-cross-windows:
	cargo install cross
	rustup target add x86_64-pc-windows-gnu
	cross build --target=x86_64-pc-windows-gnu

install:
	cp target/$(target)/$(prog) ~/bin/$(prog)-$(extension)

all: test build install

help:
	@echo "usage: make $(prog) [debug=1]"