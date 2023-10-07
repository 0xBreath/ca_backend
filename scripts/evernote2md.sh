#!/bin/bash

input=

usage() {
  if [[ -n $1 ]]; then
    echo "$*"
    echo
  fi
  cat <<EOF

usage: $0 [OPTIONS] [ARGS]

Converts Evernotes .enex file to .md file.

ARGS:
  --input               - Input file path .enex



EOF
  exit 1
}

while [[ -n $1 ]]; do
  if [[ ${1:0:1} = - ]]; then
    if [[ $1 = --input ]]; then
      input="$2"
      shift 2
    elif [[ $1 = -h ]]; then
      usage "$@"
    else
      echo "Unknown argument: $1"
      exit 1
    fi
  else
    shift
  fi
done

if [[ -z $input ]]; then
  usage "Error: --input not specified"
fi


WORKDIR="$(git rev-parse --show-toplevel)"
evernote2md "$HOME"/LIFE/C-Archive/Evernote_enex/"$input" "$WORKDIR"/data/articles