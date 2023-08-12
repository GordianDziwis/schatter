dev-run: run-debug-pi-test

# Pi Debug
run-debug-pi-test: push-debug-pi run-pi-test

run-debug-pi-stream: push-debug-pi run-pi-stream

push-debug-pi: build-debug-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/debug/schatter-client raspberrypi-1:

build-debug-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

# Pi release
run-release-pi: push-release-pi run-pi

push-release-pi: build-release-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/release/schatter-client raspberrypi-1:

build-release-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --release --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

# Local
run-debug-local:
	RUST_BACKTRACE=1 cargo run -p schatter-client stream

run-release-local:
	RUST_BACKTRACE=1 cargo run --release -p schatter-client stream

# Common
run-pi-test:
	ssh -t raspberrypi-1 'RUST_BACKTRACE=1 ./schatter-client test'

run-pi-stream:
	ssh -t raspberrypi-1 'RUST_BACKTRACE=1 ./schatter-client stream'

clean-pi:
	ssh raspberrypi-1 'rm -f schatter-client'

clean:
	cargo clean
