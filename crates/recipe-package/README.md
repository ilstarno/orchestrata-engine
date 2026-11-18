# Recipe packages v1

`orchestrata-recipe-package` is a shared, in-memory Rust loader for `.orcpkg` files. It depends on the existing mathematical recipe runtime. Loose XML APIs remain available unchanged. Descriptive and legacy assignment XML are not automatically converted to mathematical models.

```rust,ignore
let package = orchestrata_recipe_package::load_package(&archive_bytes)?;
let compiled = package.compile(None, &records_json)?; // declared default
let result = compiled.solve()?;
```

The `solve-package` executable accepts a package path, a JSON records path and an optional declared entrypoint ID. It has no HTTP interface. Compilation and solving fully recompute the model; updates are not incremental.

## Manifest and execution

The root `recipe.xml` declares format version 1, stable package ID, numeric `major.minor.patch` version, default locale, default entrypoint, author, SPDX license expression with a local XML terms reference, independent free/paid distribution tier, localized titles/descriptions, exact assets and named model entrypoints. See `tests/fixtures/manifest.xml` for the complete contract. English and Albanian translations are required. Every asset has exact expanded byte length and SHA-256 digest. Every model asset must have an entrypoint. Additional assets are local XML terms only in v1.

The entire inventory and every model are validated before entrypoints are exposed. Only a manifest-selected model is compiled. No external imports, extraction, network fetches, scripts, shell, DTDs or entity resolution are supported. The mathematical runtime retains strict named record validation and its supported operators: reference assignment over explicit eligibility edges, capacity and availability constraints, lexicographic maximum assignment count followed optionally by minimum one declared edge sum. Unsupported semantics fail closed; this is not a general mathematical programming language.

Author and license are declarations, not verified publisher identity or legal authority. A paid tier is neither an entitlement nor proof of authorization. Callers must apply their own authorization before invoking execution. There is no signature, entitlement, authentication or payment implementation here.

## Bounds and ZIP profile

At most 524288 archive bytes, 16 members, 32768 expanded bytes per member and 262144 total expanded bytes. Actual expanded bytes are counted. Paths are lowercase ASCII relative `.xml` names, at most 128 bytes and four segments. Duplicates, case aliases, traversal, devices, directories, executable permissions and symlinks are rejected.

V1 accepts classic single-disk ZIP with Stored or Deflated members. Encryption, ZIP64, extra fields, comments, self-extracting prefixes, trailing bytes, overlapping entries and mismatched headers are rejected. This intentionally bounded ZIP profile does not accept every valid ZIP producer's optional extensions. No members are extracted to disk.

Metadata has bounded text and entry counts. Package versions do not support prerelease/build suffixes. Locale codes are two lowercase letters with an optional uppercase two-letter region. XML is UTF-8, namespace-free and structurally bounded. Digests detect changed bytes; they are not signatures.

## Immutable identities

Content identity is SHA-256 of `orcpkg-content-v1\0`, a big-endian u64 member count, then all validated members sorted by path. Each member contributes big-endian u64 path length, path bytes, big-endian u64 content length and exact expanded content bytes. The root manifest is included. ZIP ordering, compression and timestamps are excluded; display metadata changes content identity.

Execution identity is SHA-256 of `orcpkg-execution-math-v1\0`, then the selected entrypoint ID, selected path and exact model bytes, each prefixed by its big-endian u64 byte length. V1 has no imports, so these are all executable bytes. Display metadata is excluded. This identity does not identify records, engine binary, authorization or an execution result; callers must retain those separately when caching or auditing results.

## Verification

`cargo test -p orchestrata-recipe-package` checks multiple executable XML models, exact synthetic results, independent identity vectors, metadata separation, changed inputs/models and conservative malformed/archive-bound rejection. `cargo run -p orchestrata-recipe-package --bin solve-package -- tests/fixtures/maintenance.orcpkg tests/fixtures/inputs/maintenance.json cost` runs the sample when invoked from this crate directory.

## Author a package

Build the validator and solver from the Engine workspace root:

```sh
cargo build --locked -p orchestrata-recipe-package --bins
python3 crates/recipe-package/tools/pack.py /path/to/source /path/to/maintenance.orcpkg --validator target/debug/validate-package
target/debug/solve-package /path/to/maintenance.orcpkg /path/to/records.json cost
```

The authoring command requires Python 3.10+ on POSIX. Use a dedicated directory containing only `recipe.xml` and its declared XML assets. Do not put inputs, notes, README files or generated archives inside that directory. Keep the source directory stable while packing; this is a local authoring tool, not a hostile multi-user filesystem service.

Start from the sample manifest. Edit its English and Albanian titles/descriptions, author and license declarations deliberately. The sample license terms are English only; translated display metadata does not translate or change the license. The fixture author explicitly marks synthetic test data with an unspecified publisher; it is not release metadata. The derived fixture retains upstream Apache-2.0 terms and provenance; no new license is granted by its metadata. No author, license or paid-tier declaration grants server access.

The packer checks every directory entry, rejects symlinks/nonregular files, hidden or undeclared files and noncanonical names, and bounds depth, count and bytes. It refreshes declared asset sizes and SHA-256 values in the packaged manifest; the source manifest is unchanged. It preserves model and terms bytes. It writes sorted Stored ZIP members with fixed timestamps and regular 0644 attributes rather than copying filesystem permissions. Rebuilding unchanged source yields identical archive bytes.

The shared Rust validator checks the resulting archive before an exclusive atomic write publishes it. Existing output paths are never overwritten. A sibling temporary file is removed on success or failure. No shell is invoked and no network is used; `--validator` must identify the trusted locally built executable. The directory snapshot is bounded but is not a transaction across concurrent source edits.

To test the authoring workflow:

```sh
VALIDATE_PACKAGE="$PWD/target/debug/validate-package" SOLVE_PACKAGE="$PWD/target/debug/solve-package" python3 crates/recipe-package/tools/test_pack.py
```

The authoring command accepts an optional leading XML 1.0 declaration with UTF-8 encoding (or omitted encoding), including an optional UTF-8 BOM. Other processing instructions, DTDs and non-UTF-8 bytes remain rejected. The output manifest is serialized as UTF-8 without a declaration.

Atomic no-clobber publication uses a hard link from a sibling temporary file. The destination filesystem must support hard links. If it does not, authoring fails without publishing the output; there is no non-atomic fallback.
