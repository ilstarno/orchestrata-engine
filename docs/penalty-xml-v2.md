# Mathematical XML v2: explicit service and omission costs

Version 1 remains unchanged: maximum assigned count followed optionally by minimum edge sum, with the existing result fields and execution identity. The separate v2 API implements one different, opt-in objective. No v1 XML is upgraded, rewritten or reinterpreted.

```xml
<model version="2" id="contact-center-penalties">
  <!-- Explicit typed inputs, parameters, decision and constraints precede this section. -->
  <objectives order="single">
    <minimize variable="routing" operator="service-plus-unassigned-cost"
      service-field="service_cents" penalty-field="missed_service_cents" unit="cents"/>
  </objectives>
</model>
```

A complete synthetic regression model is in `crates/recipe-runtime/tests/fixtures/contact-center-v2.xml`, with records in `contact-center.json` beside it. Both cost bindings are mandatory. The service field must be a declared u32 field on the decision's eligibility table. The penalty field must be a declared u32 field on the decision's item table. Every record must supply all declared fields. Omitted, null, negative, wrongly typed, unknown or oversized values fail validation; zero is accepted only when explicitly supplied. As in v1, this bounded XML profile accepts integers only through 1000000 despite the `u32` field type name. There is no penalty inferred from priority labels, names or missing values.

`unit` is one required identifier declaring the common unit of both costs. The runtime performs no conversion and cannot verify a source's physical unit from numbers. Separate service/penalty-unit attributes, multiple objectives, coverage-first mixtures, edge-sum substitutions and unknown policies are rejected. Calling code must supply values in the declared unit. The objective is exactly selected service costs plus one declared penalty per omitted item. Assigned count is reported but not optimized separately or used as a hidden tie preference.

V2 supports the existing reference decision, explicit eligibility edges, capacity and availability constraints. It requires explicit `nullable="true"`; mandatory-job semantics are not implemented and `nullable="false"` is rejected. No maximum coverage or infeasibility claim is inferred from omissions. Zero capacity, unavailability or no eligible edge can leave requests unassigned; an eligible request can also be omitted when that costs less. Tied optima have no fairness or coverage guarantee. Identical ordered inputs replay deterministically; input permutations are not promised to choose the same tied plan.

## Separate Rust and CLI boundary

Runtime methods:

- `parse_penalty_model(xml) -> PenaltyModelSpec`
- `compile_penalty_model(&spec, records) -> CompiledPenaltyModel`
- `solve_penalty_model(compiled) -> PenaltyModelSolution`
- `run_penalty_model(xml, records)` convenience composition

Package method: `ValidatedPackage::compile_penalty(selection, records) -> CompiledPenaltyPackage`, followed by `solve()`. `selection=None` means exactly the declared default entrypoint; no version fallback or policy guessing occurs. Existing `compile` and `solve-package` accept only v1 models. The separate `solve-penalty-package` CLI accepts only v2. A package can contain independently declared versions; its entire inventory is validated before any entrypoint is exposed, and the selected version must match the explicit API.

The container remains `.orcpkg` format version 1. Only the mathematical model and result contract are v2. The result envelope retains `content_identity`, `execution_identity`, and `solution`; the v2 solution contains:

```json
{
  "schema_version": 2,
  "objective": "service-plus-unassigned-cost",
  "unit": "cents",
  "assigned": 3,
  "service_cost": 600,
  "unassigned_cost": 13100,
  "total_cost": 13700,
  "assignments": [{"item_id":"fraud","resource_id":"generalist"}]
}
```

The abbreviated assignments above show one record; actual results contain every item, in input order, with stable IDs and null for omissions. No `minimized_edge_sum` alias or automatic reason code is emitted. These totals fit u64. V1 result bytes remain unchanged.

Content identity retains the existing sorted-member domain. V2 execution identity uses SHA-256 with domain `orcpkg-execution-math-v2-penalty\0`, followed by big-endian u64 length-prefixed entrypoint ID, model path and exact model bytes. Units and bindings are included via those bytes. Display metadata is excluded from execution identity. Input records, solver binary version and authorization are not bound; consumers must retain them separately. Existing v1 identity domain and values do not change.

No backend route, connector adapter, payment entitlement or deployed authorization is added. This is bounded exact assignment and full re-solving, not live routing, scheduling, incremental updates or a scale claim.

## Validation and execution are separate

Successful `validate-package` or `load_package` validates the complete archive and its declared models; it does not mean the selected entrypoint is executable by the v1 API/CLI, a hosted service, or an authorized session. Select the explicitly supported versioned execution API. No route, registration, entitlement or hosted execution is provided by validation.

With at most 32 items and each service/omission cost bounded by 1000000, service_cost, unassigned_cost and total_cost are each at most 32000000. Every item contributes exactly one service or omission cost. Those integers are exact within JavaScript's safe integer range; implementations must retain the documented bounds and exact integer arithmetic when serializing.
