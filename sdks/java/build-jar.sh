#!/bin/sh
set -eu

if [ "$#" -gt 1 ]; then
    printf 'Usage: %s [output.jar]\n' "$0" >&2
    exit 2
fi
sdk=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
output="${1:-$sdk/build/orchestrata.jar}"
case "$output" in *.jar) ;; *) printf 'Output must end with .jar\n' >&2; exit 2 ;; esac
if [ -n "${JAVA_HOME:-}" ]; then
    javac="$JAVA_HOME/bin/javac"
    jar="$JAVA_HOME/bin/jar"
else
    javac=javac
    jar=jar
fi
if ! command -v "$javac" >/dev/null || ! command -v "$jar" >/dev/null; then
    printf 'Set JAVA_HOME to a JDK 22 or newer. A Java runtime alone cannot build the JAR.\n' >&2
    exit 1
fi
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
mkdir "$tmp/classes"
"$javac" --release 22 -d "$tmp/classes" "$sdk"/src/main/java/io/orchestrata/*.java
"$jar" --create --file "$tmp/orchestrata.jar" -C "$tmp/classes" .
mkdir -p -- "$(dirname -- "$output")"
mv -f -- "$tmp/orchestrata.jar" "$output"
printf '%s\n' "$output"
