#!/bin/bash -e

CMD="cargo build --release"
echo "Running RUSTFLAGS=$RUSTFLAGS$CMD"
$CMD
