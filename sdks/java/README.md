# Java SDK

Use a 64-bit JDK 22 or newer. Build the JAR from the engine checkout:

```sh
sh sdks/java/build-jar.sh
```

The command writes `sdks/java/build/orchestrata.jar`. Pass a path ending in
`.jar` to use another location. The JAR contains the Java binding only.
Build the engine library with `cargo build --release --locked -p orchestrata-ffi`
and pass the resulting library path to `Solver`.

Compile your app with the JAR on the classpath. For example, on Linux or macOS:

```sh
javac -cp sdks/java/build/orchestrata.jar App.java
java --enable-native-access=ALL-UNNAMED -cp sdks/java/build/orchestrata.jar:. App
```

Use `;` instead of `:` in the classpath on Windows. To build without the shell
script, compile both sources in `src/main/java/io/orchestrata` with
`javac --release 22 -d classes`, then run
`jar --create --file orchestrata.jar -C classes .`.

Use `Solver` in a try-with-resources block and call `solveJson` with JSON and
the evaluation limit. The response is raw JSON. Use an integer-preserving
parser when reading numeric values. A search-budget rejection is status 2;
it is not the same as a successful JSON response with `"solution":null`.
