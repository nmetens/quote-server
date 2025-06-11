#!/bin/bash

set -e
source "$HOME/.cargo/env"

# Start back-end server in background
cd back-end
cargo run --release & # & runs in background

# Start front-end (runs in foreground)
cd ../front-end
trunk serve
