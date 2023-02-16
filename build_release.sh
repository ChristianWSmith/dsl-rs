#!/bin/bash

./format.sh
cargo build --bin client --release
cargo build --bin server --release
