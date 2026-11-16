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
