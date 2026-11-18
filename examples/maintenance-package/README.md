# Maintenance allocation from named records

A twelve-order synthetic shift demonstrates reusable mathematical XML, independent of how records arrive. Six technicians have availability and order-count capacities; twenty eligibility edges declare which assignments are allowed. Two orders cannot be assigned: the only eligible high-voltage technician is unavailable, and the lift order has no eligible technician.

## Run

Clone `ilstarno/orchestrata-engine` and `ilstarno/orchestrata-recipes` into sibling directories. From the Engine root:

```sh
cargo build --locked -p orchestrata-recipe-package --bins
PACKAGE=../orchestrata-recipes/packages/maintenance-shift/maintenance-shift.orcpkg
target/debug/validate-package "$PACKAGE"
target/debug/solve-package "$PACKAGE" examples/maintenance-package/inputs.json cost
target/debug/solve-package "$PACKAGE" examples/maintenance-package/inputs.json delay
python3 examples/maintenance-package/verify.py "$PWD/target/debug/solve-package" "$PACKAGE" "$PWD/crates/recipe-package/tools/pack.py" "$PWD/target/debug/validate-package"
```

The exact independent enumeration proves a unique optimum for each objective: **10 orders / 14600 cents** for `cost`, and **10 orders / 292 aggregate delay minutes** for `delay`. The assignments differ. Delay is a sum of supplied per-assignment estimates, not elapsed shift length or a generated schedule. All work is treated as one capacity slot; travel, durations, deadlines and precedence are not modeled.

The verification also changes capacity, availability and model objective, rejects malformed types and duplicate edges, rebuilds identical archive bytes and compares both source channels. No external service, credentials or network is used.

## Source-neutral input contract

Package ID `maintenance-shift`, version `1.0.0`; default entrypoint `cost`, alternative `delay`. The package manifest and mathematical XML are authoritative. Both entrypoints take the same exact named records:

| Table | Fields and types |
|---|---|
| `work_orders` | `id`: nonempty stable identifier string |
| `technicians` | `id`: identifier; `capacity_slots`: unsigned 32-bit integer; `available`: boolean |
| `eligibility` | `work_order_id`, `technician_id`: references; `cost_cents`, `delay_minutes`: unsigned 32-bit integers |

Extra fields, invalid types, missing references and duplicate IDs/edges are rejected by the runtime. Limits include 128 total records and 32 orders/resources. Each technician can receive at most `min(capacity_slots, 3)` orders. Only listed eligibility edges are allowed; availability and capacity are hard constraints. Maximum assignment count precedes the selected secondary sum lexicographically. Input order is preserved in the returned order list; stable IDs must come from the source adapter, not display labels or row positions.

The local adapter example accepts CSV tables or a connector-style JSON envelope of `tables -> table name -> records`. They both become the same canonical object; neither channel changes optimization behavior:

```sh
python3 examples/maintenance-package/normalize.py csv examples/maintenance-package/sources/csv > /tmp/maintenance-csv.json
python3 examples/maintenance-package/normalize.py connector examples/maintenance-package/sources/connector-records.json > /tmp/maintenance-connector.json
cmp /tmp/maintenance-csv.json /tmp/maintenance-connector.json
target/debug/solve-package "$PACKAGE" /tmp/maintenance-connector.json cost
```

This is a local normalization example, not a deployed connector. A real connector handles access, mapping and refresh outside the solver. No paid activation or server authorization is implemented here.

## Result contract

The CLI emits `content_identity`, `execution_identity`, and `solution`. The solution contains `assigned`, `minimized_edge_sum`, and `assignments` with `item_id` and nullable `resource_id`. The sum unit is cents for `cost`, minutes for `delay`. There are no per-order reason codes in this runtime; do not present inferred reasons as solver output.

Content identity binds the complete validated package. Execution identity binds the selected model and entrypoint; it differs between `cost` and `delay`. Neither identity binds input records, runtime version or authorization. Consumers must retain those separately when caching results. Changed input triggers a full solve, not an incremental update.

The package and source attribution are in the Recipes repository. Models derive from the Apache-2.0 Engine assignment examples; new inputs are synthetic and the existing Apache-2.0 repository terms apply. Display metadata is English/Albanian; license terms are English only.
