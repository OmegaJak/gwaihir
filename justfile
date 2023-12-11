set windows-shell := ["cmd.exe", "/c"]

default:
  just --list

run *args:
	cargo run -p gwaihir

build *args:
	cargo build -p gwaihir {{args}}

generate-bindings:
	just crates\networking-spacetimedb/generate-bindings

# Can be used after a testnet wipe or otherwise losing the server to recreate one with the given name
publish db_name:
	just crates\spacetimedb-server/spacetime-publish {{db_name}}

install-ubuntu-deps:
	sudo apt-get install build-essential
	sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev # for tray-icon