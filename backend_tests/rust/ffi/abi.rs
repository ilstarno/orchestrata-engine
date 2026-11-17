use std::{
    ffi::{CStr, c_char},
    ptr,
};

use orchestrata_ffi::{
    Status, orchestrata_abi_version, orchestrata_free_json_v1, orchestrata_solve_json_v1,
};

const EMPTY_PROBLEM: &[u8] = br#"{"variables":[],"constraints":[]}"#;
const INFEASIBLE_PROBLEM: &[u8] = br#"{
    "variables":[{"name":"x","domain":[0]}],
    "constraints":[{"kind":"not_equal_offset","left":0,"right":0,"offset":0,"hard_weight":1}]
}"#;

#[test]
fn v1_returns_optimal_or_proven_infeasible_documents() {
    assert_eq!(orchestrata_abi_version(), 1);
    assert_eq!(
        invoke(EMPTY_PROBLEM, 1),
        Ok(r#"{"solution":{"values":[],"score":{"hard":0,"soft":0},"evaluated":1}}"#.into())
    );
    assert_eq!(
        invoke(INFEASIBLE_PROBLEM, 1),
        Ok(r#"{"solution":null}"#.into())
    );
}

#[test]
fn v1_classifies_model_errors_and_never_allocates_on_failure() {
    let invalid_model = br#"{"variables":[{"name":"x","domain":[]}],"constraints":[]}"#;
    assert_eq!(invoke(b"{", 1), Err(Status::InvalidProblem as i32));
    assert_eq!(invoke(invalid_model, 1), Err(Status::InvalidProblem as i32));
    assert_eq!(invoke(EMPTY_PROBLEM, 0), Err(Status::SearchLimit as i32));
    assert_eq!(invoke(b"", 1), Err(Status::InvalidArgument as i32));
    assert_eq!(
        invoke(&vec![b'x'; 1_048_577], 1),
        Err(Status::InvalidArgument as i32)
    );
}

#[test]
fn v1_counts_distinct_options_in_the_evaluation_budget() {
    let duplicate_options = br#"{"variables":[{"name":"job","domain":[9,1,9,1]}],"constraints":[]}"#;
    assert_eq!(
        invoke(duplicate_options, 2),
        Ok(r#"{"solution":{"values":[9],"score":{"hard":0,"soft":0},"evaluated":2}}"#.into())
    );
    assert_eq!(invoke(duplicate_options, 1), Err(Status::SearchLimit as i32));
}

#[test]
fn v1_rejects_null_arguments_and_clears_valid_result_slots() {
    let mut output = ptr::dangling_mut::<c_char>();
    // SAFETY: null requests are rejected before reading; output points to local storage.
    let status = unsafe { orchestrata_solve_json_v1(ptr::null(), 1, 1, &mut output) };
    assert_eq!(status, Status::InvalidArgument as i32);
    assert!(output.is_null());

    // SAFETY: null output is rejected without dereferencing.
    let status = unsafe {
        orchestrata_solve_json_v1(
            EMPTY_PROBLEM.as_ptr(),
            EMPTY_PROBLEM.len(),
            1,
            ptr::null_mut(),
        )
    };
    assert_eq!(status, Status::InvalidArgument as i32);

    // SAFETY: null free is a no-op.
    unsafe { orchestrata_free_json_v1(ptr::null_mut()) };
}

fn invoke(problem: &[u8], limit: u64) -> Result<String, i32> {
    let mut output = ptr::dangling_mut::<c_char>();
    // SAFETY: problem is readable and disjoint from the writable output slot.
    let status =
        unsafe { orchestrata_solve_json_v1(problem.as_ptr(), problem.len(), limit, &mut output) };
    if status != Status::Ok as i32 {
        assert!(output.is_null());
        return Err(status);
    }

    assert!(!output.is_null());
    // SAFETY: success provides a live NUL-terminated buffer; copy before releasing.
    let text = unsafe { CStr::from_ptr(output) }
        .to_str()
        .expect("native response should be valid UTF-8")
        .to_owned();
    // SAFETY: pair the owned buffer with its only release.
    unsafe { orchestrata_free_json_v1(output) };
    Ok(text)
}

#[test]
fn v1_dispatches_additive_count_value_json_and_keeps_invalid_status() {
    let request = br#"{
      "variables":[{"name":"worker","domain":[0,1]}],
      "constraints":[{"kind":"count_value","variables":[0],"value":1,"min":1,"max":1,"weight":5,"tier":"hard"}]
    }"#;
    let result: serde_json::Value = serde_json::from_str(&invoke(request, 2).unwrap()).unwrap();
    assert_eq!(result["solution"]["values"], serde_json::json!([1]));

    let mut invalid: serde_json::Value = serde_json::from_slice(request).unwrap();
    invalid["constraints"][0]["max"] = serde_json::json!(2);
    assert_eq!(
        invoke(&serde_json::to_vec(&invalid).unwrap(), 2),
        Err(Status::InvalidProblem as i32)
    );
    invalid["constraints"][0]["tier"] = serde_json::json!("unknown");
    assert_eq!(
        invoke(&serde_json::to_vec(&invalid).unwrap(), 2),
        Err(Status::InvalidProblem as i32)
    );
}

#[test]
fn v1_accepts_exclusive_intervals_and_rejects_malformed_members() {
    let json = br#"{
        "variables":[{"name":"a","domain":[0]},{"name":"b","domain":[1,2]}],
        "constraints":[{"kind":"no_overlap","intervals":[{"start":0,"duration":2},{"start":1,"duration":1}]}]
    }"#;
    let result: serde_json::Value = serde_json::from_str(&invoke(json, 2).unwrap()).unwrap();
    assert_eq!(result["solution"]["values"], serde_json::json!([0, 2]));
    let mut model: serde_json::Value = serde_json::from_slice(json).unwrap();
    for member in [
        serde_json::json!({"start":1,"duration":-1}),
        serde_json::json!({"start":0,"duration":0}),
        serde_json::json!({"start":1,"duration":1,"units":"minutes"}),
    ] {
        model["constraints"][0]["intervals"][1] = member;
        assert_eq!(
            invoke(&serde_json::to_vec(&model).unwrap(), 2),
            Err(Status::InvalidProblem as i32)
        );
    }
}
