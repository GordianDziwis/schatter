dev-run: dev-build
	ssh raspberrypi-1 rm -f schatter-client
	scp target/armv7-unknown-linux-gnueabihf/debug/schatter-client raspberrypi-1:
	ssh raspberrypi-1 ./schatter-client
	cargo run

dev-build:
	CROSS_CONTAINER_ENGINE=podman cross build --package schatter-client --target armv7-unknown-linux-gnueabihf
	cargo build --package schatter-server

clean:
	cargo clean
