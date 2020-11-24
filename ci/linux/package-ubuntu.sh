#!/bin/bash

set -e

script_dir=$(dirname "$0")

export GIT_HASH=$(git rev-parse --short HEAD)
export PKG_VERSION="1-$GIT_HASH-$BRANCH_SHORT_NAME-git"

if [[ "$BRANCH_FULL_NAME" =~ "^refs/tags/" ]]; then
	export PKG_VERSION="$BRANCH_SHORT_NAME"
fi

cd ./build

sudo chmod ao+r ../package/*
