#!/bin/bash

cargo run -r -p ca_admin -- \
  -t article \
  -f ~/LIFE/Coding/c_archive/public/markdown/kriya-yoga-intro.md \
  -n "Kriya Yoga Intro" \
  -i /images/articles/kriya-yoga-intro.png

cargo run -r -p ca_admin -- \
  -t article \
  -f ~/LIFE/Coding/c_archive/public/markdown/8_Ways_To_Dramatically_Increase_Your_Level_of_Consciousness.md \
  -n "8 Ways to Dramatically Increase Your Level of Consciousness" \
  -i /images/articles/what-is-love.png