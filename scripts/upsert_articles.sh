#!/bin/bash

WORKDIR="$(git rev-parse --show-toplevel)"

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/Understanding_the_Difference_Between_Your_Small_Self_and_Your_Higher_Self.md \
  -n "Understanding the Difference Between Your Small Self and Your Higher Self" \
  -i /images/articles/kriya-yoga-intro.png

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/8_Ways_To_Dramatically_Increase_Your_Level_of_Consciousness.md \
  -n "8 Ways to Dramatically Increase Your Level of Consciousness" \
  -i /images/articles/what-is-love.png

cargo run -r -p admin -- \
  -t article \
  -f "$WORKDIR"/data/articles/Understanding_Soul_Contracts.md \
  -n "Understanding Soul Contracts" \
  -i /images/articles/map-of-consciousness-explained.png