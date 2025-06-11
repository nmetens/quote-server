#!/bin/bash

: <<'COMMENT'
    What This Script Does:
    - Starts the backend in the background and stores its PID.
    - Uses trap to run a cleanup function when the script exits (for Ctrl+C).
    - Kills the backend process and waits for it to terminate.
    - Prevents zombie processes listening on port 8000.
    - Help from ChatGPT to set this up.
COMMENT

set -e
source "$HOME/.cargo/env"

# Get script root directory
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Start back-end server in background and capture its PID
cd "$ROOT_DIR/back-end"
echo "🚀 Starting back-end..."
cargo run --release &
BACKEND_PID=$!

# Define cleanup function
cleanup() {
  echo -e "\n🛑 Stopping back-end (PID $BACKEND_PID)..."
  kill $BACKEND_PID 2>/dev/null || true
  wait $BACKEND_PID 2>/dev/null || true
  echo "✅ Back-end stopped."
}

# Trap script exit and call cleanup
trap cleanup EXIT

# Start front-end
cd "$ROOT_DIR/front-end"
echo "🎨 Starting front-end..."
trunk serve
