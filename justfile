set windows-shell := ["cmd.exe", "/c"]

run:
	cargo run -p gwaihir

run-release:
	cargo run -p gwaihir --release

build:
	cargo build -p gwaihir

build-release:
	cargo build -p gwaihir --release

install-ubuntu-deps:
	sudo apt-get install build-essential
	sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev # for tray-icon