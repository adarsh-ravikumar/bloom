#!/bin/sh
set -e

cargo build --manifest-path bloom/Cargo.toml

cp bloom/target/debug/bloom sandbox/bloom

echo "Done!"
