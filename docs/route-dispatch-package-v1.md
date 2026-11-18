# Route-dispatch package adapter v1

This separate opt-in interface wraps the published bounded pickup/delivery constructor. It is not mathematical assignment model v1 or penalty model v2. The package container remains `.orcpkg` v1; the asset root is explicitly different:

```xml
<route-dispatch version="1" id="dispatch"
 input-schema="directed-dispatch-v1" algorithm="input-order-insertion"
 time-unit="minutes" load-unit="kg" max-insertions="200"/>
```

Only `input-order-insertion` and `two-order-insertion` are accepted. Unsupported versions, units, schemas, attributes, nested instructions and algorithms fail closed. XML contains no executable scripts, imports, external travel sources or arbitrary callbacks. Root ID is descriptive and never selects behavior. The fixed typed input schema is deliberately narrow; it does not support arbitrary field mappings inside XML.

## Accepted records and semantics

A strict JSON object contains four named collections:

- `locations`: unique stable location ID strings, at most 128.
- `couriers`: at most 16 records with `id`, `mode`, `depot`, `shift: {start,end}`, `capacity_kg`.
- `shipments`: at most 32 records with `id`, `pickup`, `delivery`, `weight_kg`, `allowed_modes`. Each stop has `location`, `window: {start,end}`, `service_minutes`.
- `travel_legs`: at most 8192 unique records with `mode`, `from`, `to`, `minutes`.

Modes are `vehicle`, `bicycle`, or `walking`. IDs contain lowercase ASCII letters, digits or hyphens and are nonempty and at most 64 bytes. All referenced locations must be explicitly declared. Unknown/duplicate fields, duplicate IDs/edges, invalid types, modes, windows and references fail validation. Input bytes are bounded at 524288 before deserialization. Times, durations and kilogram quantities are nonnegative integers at most 1000000. Shift/window starts must not exceed ends. Native validation additionally enforces supported mode sets and problem semantics.

Travel is supplied static directed mode-specific elapsed time, not geographic distance. A missing cross-location edge is unknown/unreachable; reverse edges and indirect paths are never synthesized. Self-location rows are rejected; the native API explicitly treats staying at the same location as zero travel without a lookup. No external service is called.

Couriers start empty at their supplied shift start/depot and return to that depot within their shift. Both pickup and delivery belong to the same courier, with pickup before delivery. Weight onboard must remain within capacity. Service may wait until a window starts and must finish before its end and shift end. These are exactly the native constructor's supported rules, not time-dependent traffic, breaks, dynamic arrivals, vehicle-specific access restrictions, or real maps.

## Caller budget and heuristic policy

`max-insertions` in XML is a ceiling at most 20000, not an execution default. The caller must explicitly supply a total insertion budget, including zero, and it must not exceed that ceiling. No policy increases it. The trial budget counts candidate insertion simulations; parsing and feasibility verification are additional bounded work, not counted as insertion trials.

Input-order insertion runs the existing constructor once. The optional two-order adapter first runs the identical baseline, then only if budget remains and the alternative order differs, retries from scratch ordered by delivery-window end, pickup-window end, original index. Its second pass receives exactly the remaining budget. Fewer unassigned shipments wins; equal coverage prefers lower summed depot-to-depot duration; the baseline wins an exact tie. Original shipment indices are restored before stable IDs are returned. Any native error fails the request rather than reporting an unknown trial count.

Every successful plan is natively verified for its assigned subset. Neither a completed pass nor an omitted shipment proves optimality or infeasibility. The label is always `verified-feasible-heuristic`, including when a tiny independent oracle happens to certify the same result. The reported total courier-minutes includes travel, waiting and service for used routes, not distance or just driving time.

## Explicit API, CLI and result

Runtime: `route_dispatch::{parse_route_model, compile_route_model, solve_route_model}`. Package: `ValidatedPackage::compile_route(selection, records, budget)?.solve()`. CLI:

```sh
solve-route-package package.orcpkg records.json 200 baseline
```

The optional entrypoint selects exactly a declared manifest entrypoint; omission means the manifest default. The v1 and penalty-v2 execution APIs reject route assets; route execution rejects mathematical assets. Validation alone does not imply compatibility with any selected execution API, hosted route, authorization or entitlement.

The result envelope contains `content_identity`, `execution_identity`, `input_identity`, `request_identity`, and `solution`. Typed solution fields are `schema: route-dispatch-result-v1`, `algorithm`, `guarantee`, time/load units, `supplied_max_insertions`, `evaluated_insertions`, `total_courier_minutes`, `routes`, `unassigned`, and `passes`. Each route names its courier, departure/return minutes, and ordered typed pickup/delivery visits with stable shipment/location IDs, arrival, service start and service end. Unassigned IDs retain input order; no unsupported explanation code is invented. Each pass reports order, actual trial count, native stop reason and whether selected. Counts across passes sum to the reported total. A global stop label is deliberately avoided because the selected and alternate passes may stop differently.

Identities use SHA-256. Content identity is the existing sorted-member hash. Route execution uses domain `orcpkg-execution-route-dispatch-v1\0`, then u64-BE length-prefixed entrypoint ID, model path and exact model bytes. Input identity uses `orcpkg-route-input-v1\0` plus u64-BE length-prefixed exact UTF-8 JSON bytes. It is not canonical semantic JSON: whitespace changes its identity. Request identity uses `orcpkg-route-request-v1\0`, then fixed 32-byte content, execution and input digests, then the caller budget as u32-BE. None of these identifies the runtime binary or proves authorization; retain those separately for replay/audit.

With fixed input bounds, all output clocks are at most 1000000, total courier-minutes at most 16000000, trials at most 20000 and visits at most 64. These quantities fit JavaScript safe integers without rounding. Each run fully recomputes; there are no incremental updates or scale claims.

An explicit Rust [coverage certificate](route-coverage.md) can separately bound
the number of served shipments. It does not alter these commands or result bytes,
and it does not certify courier-minutes or any other objective.
