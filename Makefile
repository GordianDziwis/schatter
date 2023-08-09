dev-run: run-debug-pi

# Debug
run-debug-pi: push-debug-pi run-pi

push-debug-pi: build-debug-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/debug/schatter-client raspberrypi-1:

build-debug-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

# Release
run-release-pi: push-release-pi run-pi

push-release-pi: build-release-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/release/schatter-client raspberrypi-1:

build-release-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --release --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

# Local
run-debug-local: build-debug-local
	RUST_BACKTRACE=1 cargo run -p schatter-client

build-debug-local:
	cargo build

# Common
run-pi:
	ssh -t raspberrypi-1 'RUST_BACKTRACE=1 ./schatter-client "$$(hostname -I | awk "{print $$1}")"'

clean-pi:
	ssh raspberrypi-1 'rm -f schatter-client'

clean:
	cargo clean
