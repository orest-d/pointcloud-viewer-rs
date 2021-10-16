#!/bin/bash
set -eu

rustup target add wasm32-unknown-unknown

# Release:
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/pointcloud-viewer.wasm assets/

wasm-strip assets/pointcloud-viewer.wasm
