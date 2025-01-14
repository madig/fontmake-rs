#!/usr/bin/env bash

set -o nounset
set -o errexit

cargo run -p fontc -- --source resources/testdata/static.designspace

cd build

OTS_VER=9.0.0
rm -f "ots-${OTS_VER}-Linux.zip"
rm -rf "ots-${OTS_VER}-Linux"
curl -OL "https://github.com/khaledhosny/ots/releases/download/v${OTS_VER}/ots-${OTS_VER}-Linux.zip"
unzip "ots-${OTS_VER}-Linux.zip" "ots-${OTS_VER}-Linux/ots-sanitize"
"ots-${OTS_VER}-Linux/ots-sanitize" font.ttf