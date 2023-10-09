#!/bin/bash

WORKDIR="$(git rev-parse --show-toplevel)"

cargo run -r -p admin -- \
  -t testimonials \
  -f "$WORKDIR"/data/testimonials/testimonials.json