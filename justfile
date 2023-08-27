set shell := ["cmd.exe", "/c"]

run:
	cargo run -p gwaihir

run-release:
	cargo run -p gwaihir --release

build:
	cargo build -p gwaihir

build-release:
	cargo build -p gwaihir --release