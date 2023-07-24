dev-run: dev-run-local

dev-run-pi: dev-build-pi
	ssh raspberrypi-1 rm -f schatter-client
	scp target/armv7-unknown-linux-gnueabihf/debug/schatter-client raspberrypi-1:
	ssh raspberrypi-1 ./schatter-client
	cargo run schatter-server

dev-run-local: dev-build-local
	cargo run -p schatter-server &
	cargo run -p schatter-client

dev-build-pi:
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

dev-build-local:
	cargo build

clean:
	cargo clean
