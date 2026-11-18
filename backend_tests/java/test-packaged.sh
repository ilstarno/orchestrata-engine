#!/bin/sh
set -eu
if [ "$#" != 1 ]; then
    printf 'Usage: %s <path-to-liborchestrata_ffi.so>\n' "$0" >&2
    exit 2
fi
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH='' cd -- "$here/../.." && pwd)
if [ -n "${JAVA_HOME:-}" ]; then
    javac="$JAVA_HOME/bin/javac"
    java="$JAVA_HOME/bin/java"
    jar="$JAVA_HOME/bin/jar"
else
    javac=javac
    java=java
    jar=jar
fi
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
binding="$tmp/binding jar/orchestrata.jar"
sh "$root/sdks/java/build-jar.sh" "$binding"
"$jar" --list --file "$binding" > "$tmp/contents"
grep -q '^io/orchestrata/Solver.class$' "$tmp/contents"
grep -q '^io/orchestrata/SolveException.class$' "$tmp/contents"
if grep -E 'Smoke\.class|\.(so|dll|dylib)$' "$tmp/contents"; then
    printf 'The SDK JAR must not include test classes or native binaries.\n' >&2
    exit 1
fi
"${CC:-cc}" -shared -fPIC "$here/incomplete_library.c" -o "$tmp/incomplete.so"
mkdir "$tmp/app"
# Compile independently with only the produced JAR as the SDK classpath.
"$javac" --release 22 -cp "$binding" -d "$tmp/app" "$here/io/orchestrata/PackagedSmoke.java" "$here/io/orchestrata/Smoke.java"
"$java" --enable-native-access=ALL-UNNAMED -cp "$binding:$tmp/app" io.orchestrata.Smoke "$1"
"$java" --enable-native-access=ALL-UNNAMED -cp "$binding:$tmp/app" io.orchestrata.PackagedSmoke "$1" "$tmp/incomplete.so"
