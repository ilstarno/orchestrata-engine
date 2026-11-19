#!/usr/bin/env sh
# Usage: sh standalone-install.test.sh NODE_SDK_DIR FFI_INCLUDE_DIR FFI_LIBRARY_DIR
# Requires Node, npm, a C toolchain and optionally npm_config_nodedir for offline build
set -eu
sdk=$(cd "$1" && pwd)
include=$(cd "$2" && pwd)
lib=$(cd "$3" && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
export npm_config_cache="$tmp/cache"
mkdir "$tmp/consumer"
packed=$(cd "$sdk" && npm pack --ignore-scripts --pack-destination "$tmp")
cd "$tmp/consumer"
printf '%s\n' '{"name":"sdk-install-test","version":"1.0.0","private":true}' > package.json
ORCHESTRATA_INCLUDE_DIR="$include" ORCHESTRATA_LIB_DIR="$lib" npm install --offline "$tmp/$packed"
LD_LIBRARY_PATH="$lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" node <<'JS'
const assert = require('node:assert/strict');
const { solveJson } = require('@orchestrata/native');
const request = '{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}';
const reply = solveJson(request, 1n);
assert.match(reply, /"values":\[9223372036854775807\]/);
assert.equal(JSON.parse(reply).solution.score.hard, 0);
JS
