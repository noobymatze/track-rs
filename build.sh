#!/usr/bin/env bash

# TODO: Maybe this can be made obsolete by using: https://github.com/cross-rs/cross/blob/main/docs/cross_toml.md#targettargetimage
# rustup target add x86_64-unknown-linux-musl
cross build --target x86_64-unknown-linux-musl --release
