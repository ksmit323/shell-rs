#!/bin/sh

set -e # Exit early if any commands fail

(
  cd "$(dirname "$0")"
  cargo build --release
)

exec target/release/shell-rs "$@"

