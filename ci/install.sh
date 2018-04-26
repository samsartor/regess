#!/usr/bin/bash
set -ex

yarn global add parcel-bundler@1.7.0

if ! cargo install --list | grep "cargo-web v0.6"; then
    cargo install cargo-web --force --version "^0.6.10"
fi
