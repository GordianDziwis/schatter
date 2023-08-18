HOSTNAME=raspberrypi-1

dev-run: run-debug-pi-test

# Pi Debug
run-debug-pi-test: push-debug-pi run-pi-test

run-debug-pi-stream: push-debug-pi run-pi-stream

push-debug-pi: build-debug-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/debug/schatter-client $(HOSTNAME):

build-debug-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target armv7-unknown-linux-gnueabihf
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target aarch64-unknown-linux-gnu
	cargo build --package schatter-server

# Pi release
run-release-pi-test: push-release-pi run-pi-test

run-release-pi-stream: push-release-pi run-pi-stream

push-release-pi: build-release-pi clean-pi
	scp ./target/armv7-unknown-linux-gnueabihf/release/schatter-client $(HOSTNAME):

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
	ssh -t $(HOSTNAME) 'RUST_BACKTRACE=1 ./schatter-client test'

run-pi-stream:
	ssh -t $(HOSTNAME) 'RUST_BACKTRACE=1 ./schatter-client stream'

clean-pi:
	ssh $(HOSTNAME) 'rm -f schatter-client'

clean:
	cargo clean
