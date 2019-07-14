#!/usr/bin/env sh
set -ex
VERSION="$1"
sed -i.bak -E "s/(version = \")([0-9]+.[0-9]+.[0-9]+)(\".+# !V$)/\1$VERSION\3/g" {.,*}/Cargo.toml
rm -rfv {.,*}/Cargo.toml.bak
git diff {.,*}/Cargo.toml
