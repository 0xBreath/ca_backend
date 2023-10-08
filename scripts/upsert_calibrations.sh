#!/bin/bash

WORKDIR="$(git rev-parse --show-toplevel)"

cargo run -r -p admin -- \
  -t calibrations \
  -f "$WORKDIR"/data/calibrations/movies.json

cargo run -r -p admin -- \
  -t calibrations \
  -f "$WORKDIR"/data/calibrations/sports.json

cargo run -r -p admin -- \
  -t calibrations \
  -f "$WORKDIR"/data/calibrations/books.json

cargo run -r -p admin -- \
  -t calibrations \
  -f "$WORKDIR"/data/calibrations/people.json