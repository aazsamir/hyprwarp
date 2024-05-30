#!/bin/sh

cargo build --release

cp target/release/hyprwarp ~/.local/bin/hyprwarp
