#!/bin/bash

WORKDIR="$(git rev-parse --show-toplevel)"

cargo run -r -p admin -- \
  -t articles \
  -f "$WORKDIR"/data/articles/articles.json