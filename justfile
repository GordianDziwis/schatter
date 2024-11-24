subcommand := 'test'
build_conf := 'debug'
build_conf_str := if build_conf == "debug" { "" } else { "--release" }
debug_str := if build_conf == "debug" { "RUST_BACKTRACE=1" } else { "" }

run-local: (build 'x86_64-unknown-linux-gnu') run-server run-client-local

run-clients: run-server (run-client-pi "armv7-unknown-linux-gnueabihf" "raspberrypi-1") (run-client-pi "aarch64-unknown-linux-gnu" "raspberrypi-2")

run-client-pi target hostname: (push-pi target hostname)
    ssh -t {{ hostname }} 'sudo killall schatter-client || true'
    i3-sensible-terminal -e "ssh -t {{ hostname }} '{{ debug_str }} sudo ./schatter-client {{ subcommand }} 34254 18 9'" &
    i3-sensible-terminal -e "ssh -t {{ hostname }} '{{ debug_str }} sudo ./schatter-client {{ subcommand }} 34255 21 10'" &


run-client-local:
    {{ debug_str }} cargo run -p schatter-client stream &

run-server:
    i3-sensible-terminal -e "cargo run {{ build_conf_str }} -p schatter-server" &
    echo "Press enter to close server"

push-pi target hostname: (clean-pi hostname) (build target)
    scp target/{{ target }}/{{ build_conf }}/schatter-client {{ hostname }}:

build target:
    cargo build {{ build_conf_str }} --package schatter-server
    CROSS_CONTAINER_ENGINE=podman cross build {{ build_conf_str }} --package schatter-client {{ if target == "" { "" } else { "--target " + target  } }}

clean-pi hostname:
    ssh {{ hostname }} 'rm -f schatter-client'

clean:
    cargo clean
