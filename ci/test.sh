#!/bin/bash -e
cargo clippy --all-targets
cargo test
