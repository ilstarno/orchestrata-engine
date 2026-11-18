# Recipe data and Rust consumers

The recipes repository supplies XML data. Rust consumers use this repository.

- `crates/recipe-runtime`: executable assignment and mathematical XML compilation.
- `crates/recipe-catalog`: the compatible `orchestrata-recipes` Rust package, catalog parser,
  payload validation, synthetic workload generator and embedded legacy catalog snapshots.
- `examples/assignment-runner`: the existing command-line assignment example.

For a Git dependency, retain the Rust package name `orchestrata-recipes`, change the repository
URL to `https://github.com/ilstarno/orchestrata-engine.git`, and pin a reviewed commit containing
`crates/recipe-catalog`. Cargo locates the package within this repository. Local dependencies
can use `crates/recipe-catalog` as their path. Existing recipe IDs, exported types and functions,
logical example paths, and bundled legacy JSON examples remain available through that package.
The assignment compiler is re-exported from `orchestrata-recipe-runtime` and rejects positional
records where named objects are required.

From this checkout:

```sh
cargo test --locked -p orchestrata-recipes
cargo test --locked --manifest-path examples/assignment-runner/Cargo.toml
cargo run --locked --manifest-path examples/assignment-runner/Cargo.toml -- \
  crates/recipe-catalog/fixtures/recipes/executable/field-service/recipe.xml \
  crates/recipe-catalog/fixtures/recipes/executable/field-service/inputs.json
cargo run --locked -p orchestrata-recipe-runtime --bin solve-model -- \
  crates/recipe-catalog/fixtures/recipes/mathematical/maintenance-cost.xml \
  crates/recipe-runtime/tests/fixtures/math/records.json
```

Catalog snapshots preserve compatibility; they are not a claim that descriptive XML executes.
For new computation use the mathematical model or executable assignment APIs. Input JSON and
expected outputs belong with the consuming application or engine tests, not the XML data repository.
Existing consumers pinned to an older recipes commit continue to use that historical source.
Consumers tracking a mutable recipes branch must migrate their Cargo dependency to this repository.
Unknown external consumers cannot be discovered exhaustively; the data-only repository is not a
Cargo package. Update dependencies before advancing such consumers to a new recipes revision.
