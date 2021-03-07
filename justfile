readme:
    cargo readme > README.md

run-example example="hello_world": build-adapters
    cargo run --package dynamite --example {{example}}

build: build-adapters
    cargo build

build-adapters: build-adapters-python

build-adapters-python:
    cargo build --package dynamite_python