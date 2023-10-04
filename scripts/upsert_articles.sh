#!/bin/bash

WORKDIR="$(git rev-parse --show-toplevel)"

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/Understanding_the_Difference_Between_Your_Small_Self_and_Your_Higher_Self.md \
  -n "Understanding the Difference Between Your Small Self and Your Higher Self" \
  -i https://storage.cloud.google.com/consciousness-archive/images/articles/Understanding_the_Difference_Between_Your_Small_Self_and_Your_Higher_Self.png

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/8_Ways_To_Dramatically_Increase_Your_Level_of_Consciousness.md \
  -n "8 Ways to Dramatically Increase Your Level of Consciousness" \
  -i https://storage.cloud.google.com/consciousness-archive/images/articles/8_Ways_To_Dramatically_Increase_Your_Level_of_Consciousness.png

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/Understanding_Soul_Contracts.md \
  -n "Understanding Soul Contracts" \
  -i https://storage.cloud.google.com/consciousness-archive/images/articles/Understanding_Soul_Contracts.png
