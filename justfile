# Default values
hostname := 'raspberrypi-1'
subcommand := 'test'
target := 'debug'

run-pi target=target hostname=hostname subcommand=subcommand: (clean-pi hostname) (push-pi target hostname)
    ssh -t {{ hostname }} '{{ if target == "debug" {"RUST_BACKTRACE=1"} else {""} }} ./schatter-client {{ subcommand }}'

push-pi target hostname: (build-pi target)
    scp target/armv7-unknown-linux-gnueabihf/{{ target }}/schatter-client {{ hostname }}:

run-local subcommand target:
    {{ if target == "debug" {"RUST_BACKTRACE=1"} else {""} }} cargo run --{{ target }} -p schatter-client {{ subcommand }}


build-pi target:
    CROSS_CONTAINER_ENGINE=podman cross build {{ if target == "release" {"--release"} else {""} }} --package schatter-client --target armv7-unknown-linux-gnueabihf
    cargo build {{ if target == "release" {"--release"} else {""} }} --package schatter-server

clean-pi hostname:
    ssh {{ hostname }} 'rm -f schatter-client'

clean:
    cargo clean
