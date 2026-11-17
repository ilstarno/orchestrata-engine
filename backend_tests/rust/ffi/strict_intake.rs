use std::{
    ffi::{CStr, c_char},
    ptr,
};

use orchestrata_ffi::{
    Status, StrictStatus, orchestrata_free_json_v1, orchestrata_solve_json_strict_v1,
    orchestrata_solve_json_v1,
};

fn invoke(request: &str, budget: u64, strict: bool) -> Result<String, i32> {
    let solve = if strict {
        orchestrata_solve_json_strict_v1
    } else {
        orchestrata_solve_json_v1
    };
    let mut result = ptr::dangling_mut::<c_char>();
    // SAFETY: the request is readable and disjoint from the writable result slot.
    let status = unsafe { solve(request.as_ptr(), request.len(), budget, &mut result) };
    if status != Status::Ok as i32 {
        assert!(result.is_null());
        return Err(status);
    }
    assert!(!result.is_null());
    // SAFETY: a successful result is a live, NUL-terminated UTF-8 buffer.
    let text = unsafe { CStr::from_ptr(result) }
        .to_str()
        .unwrap()
        .to_owned();
    // SAFETY: transfer the result to its matching deallocator exactly once.
    unsafe { orchestrata_free_json_v1(result) };
    Ok(text)
}

#[test]
fn strict_preserves_the_full_result_for_all_supported_constraints() {
    let requests = [
        r#"{"variables":[{"name":"x","domain":[-9223372036854775808,9223372036854775807,9223372036854775807]}],"constraints":[{"kind":"prefer_value","variable":0,"value":9223372036854775807,"soft_weight":1}]}"#,
        r#"{"variables":[{"name":"a","domain":[0,1]},{"name":"b","domain":[0,1]}],"constraints":[{"kind":"all_different","variables":[0,1],"hard_weight":1},{"kind":"count_value","variables":[0,1],"value":0,"min":1,"max":1,"weight":1,"tier":"hard"},{"kind":"precedence","before":0,"after":1,"min_gap":1},{"kind":"not_equal_offset","left":0,"right":1,"offset":0,"hard_weight":1},{"kind":"no_overlap","intervals":[{"start":0,"duration":1},{"start":1,"duration":1}]},{"kind":"resource_capacity","items":[{"variable":0,"load":1},{"variable":1,"load":1}],"limits":[{"resource":0,"capacity":1},{"resource":1,"capacity":1}]}]}"#,
    ];
    for request in requests {
        let legacy = invoke(request, 64, false).unwrap();
        assert!(legacy.contains(r#""solution":{"values""#), "{legacy}");
        assert_eq!(invoke(request, 64, true), Ok(legacy));
    }
}

#[test]
fn legacy_unknown_fields_and_positional_objects_are_unchanged() {
    for request in [
        r#"{"variables":[],"constraints":[],"note":"legacy extra field"}"#,
        r#"[[],[]]"#,
        r#"{"variables":[["x",[0]]],"constraints":[]}"#,
    ] {
        assert!(invoke(request, 1, false).is_ok(), "{request}");
        assert!(invoke(request, 1, true).is_err(), "{request}");
    }
}

#[test]
fn strict_identifies_unknown_fields_across_wire_objects() {
    for request in [
        r#"{"variables":[],"constraints":[],"note":true}"#,
        r#"{"variables":[{"name":"x","domain":[0],"note":true}],"constraints":[]}"#,
        r#"{"variables":[],"constraints":[{"kind":"prefer_value","variable":0,"value":0,"soft_weight":1,"hard_weight":1}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"no_overlap","intervals":[{"start":0,"duration":1,"extra":true}]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[{"variable":0,"load":1,"extra":true}],"limits":[]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[],"limits":[{"resource":0,"capacity":1,"extra":true}]}]}"#,
    ] {
        assert_eq!(
            invoke(request, 0, true),
            Err(StrictStatus::UnknownField as i32),
            "{request}"
        );
    }
}

#[test]
fn strict_rejects_duplicate_including_escaped_keys_at_every_depth() {
    for request in [
        r#"{"variables":[],"variabl\u0065s":[],"constraints":[]}"#,
        r#"{"variables":[{"name":"x","name":"y","domain":[0]}],"constraints":[]}"#,
        r#"{"variables":[],"constraints":[{"kind":"no_overlap","kind":"no_overlap","intervals":[]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"no_overlap","intervals":[{"start":0,"duration":1,"duration":1}]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[{"variable":0,"load":1,"load":1}],"limits":[]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[],"limits":[{"resource":0,"capacity":1,"capacity":1}]}]}"#,
    ] {
        assert_eq!(
            invoke(request, 0, true),
            Err(StrictStatus::DuplicateField as i32),
            "{request}"
        );
    }
}

#[test]
fn strict_requires_named_structures_and_distinguishes_failures_before_search() {
    for request in [
        r#"[[],[]]"#,
        r#"{"variables":[["x",[0]]],"constraints":[]}"#,
        r#"{"variables":[],"constraints":[["no_overlap",[]]]}"#,
        r#"{"variables":[],"constraints":[{"kind":"no_overlap","intervals":[[0,1]]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[[0,1]],"limits":[]}]}"#,
        r#"{"variables":[],"constraints":[{"kind":"resource_capacity","items":[],"limits":[[0,1]]}]}"#,
        r#"{"variables":[{"name":"x","domain":[1.0]}],"constraints":[]}"#,
        r#"{"variables":[]}"#,
        r#"{"variables":[],"constraints":[{"kind":"unrecognized"}]}"#,
    ] {
        assert_eq!(
            invoke(request, 0, true),
            Err(StrictStatus::InvalidShape as i32),
            "{request}"
        );
    }
    assert_eq!(
        invoke("{", 0, true),
        Err(StrictStatus::MalformedJson as i32)
    );
    assert_eq!(
        invoke(r#"{"variables":[],"constraints":[]}[]"#, 0, true),
        Err(StrictStatus::MalformedJson as i32)
    );
    assert_eq!(
        invoke(
            r#"{"variables":[{"name":"x","domain":[]}],"constraints":[]}"#,
            0,
            true
        ),
        Err(Status::InvalidProblem as i32)
    );
    assert_eq!(
        invoke(r#"{"variables":[],"constraints":[]}"#, 0, true),
        Err(Status::SearchLimit as i32)
    );
}
