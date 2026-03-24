#!/bin/sh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# configuration
MODULE_NAME="recent"
RELEASE_DIR=/usr/local/bin

# build
(cd ${SCRIPT_DIR}/.. && cargo build --release)

# deploy
cp -p ${SCRIPT_DIR}/../target/release/$MODULE_NAME $RELEASE_DIR
