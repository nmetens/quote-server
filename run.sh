#!/bin/bash

. "$HOME/.cargo/env" # Gets cargo installed in dir.

cargo build --release # Build the project.

trunk serve # Run the leptos front-end service.
