#!/bin/bash

./format.sh
cargo build --bin client
cargo build --bin server
