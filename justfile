# Default values
hostname := 'raspberrypi-1'
subcommand := 'test'
build_conf := 'debug'
target := 'aarch64-unknown-linux-gnu'

run-pi build_conf=build_conf target=target hostname=hostname subcommand=subcommand: (clean-pi hostname) (push-pi build_conf target hostname)
    ssh -t {{ hostname }} '{{ if build_conf == "debug" {"RUST_BACKTRACE=1"} else {""} }} sudo ./schatter-client {{ subcommand }}'

push-pi build_conf target hostname: (build-pi build_conf target)
    scp target/{{ target }}/{{ build_conf }}/schatter-client {{ hostname }}:

run-local subcommand build_conf:
    {{ if build_conf == "debug" {"RUST_BACKTRACE=1"} else {""} }} cargo run --{{ build_conf }} -p schatter-client {{ subcommand }}


build-pi build_conf target:
    CROSS_CONTAINER_ENGINE=podman cross build {{ if build_conf == "release" {"--release"} else {""} }} --package schatter-client --target {{ target }}
    cargo build {{ if build_conf == "release" {"--release"} else {""} }} --package schatter-server

clean-pi hostname:
    ssh {{ hostname }} 'rm -f schatter-client'

clean:
    cargo clean
