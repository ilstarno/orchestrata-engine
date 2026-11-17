use std::{
    ffi::{CString, c_char},
    panic::catch_unwind,
    ptr, slice,
};

use orchestrata_solver::{Error, PreparedProblem, Problem};

use crate::{
    constants::{ABI_VERSION, MAX_REQUEST_BYTES, MAX_RESULT_BYTES},
    request::StrictRequest,
    result_document::ResultDocument,
    status::Status,
};

#[unsafe(no_mangle)]
pub extern "C" fn orchestrata_abi_version() -> u32 {
    ABI_VERSION
}

/// Parses and solves a standalone engine problem encoded as UTF-8 JSON.
///
/// # Safety
///
/// The output slot must be writable, aligned, and disjoint from the request.
/// For an accepted length, the request pointer must be readable for that many
/// bytes and no alias may mutate them during the call. Neither pointer is saved.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orchestrata_solve_json_v1(
    request: *const u8,
    request_length: usize,
    max_evaluations: u64,
    result: *mut *mut c_char,
) -> i32 {
    // SAFETY: this function has the same pointer contract as solve_request.
    unsafe {
        solve_request(
            request,
            request_length,
            max_evaluations,
            result,
            decode_legacy,
        )
    }
}

/// Opt-in JSON intake with named objects and no unknown or duplicate fields.
/// # Safety
/// The pointer and ownership contract is identical to [orchestrata_solve_json_v1].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orchestrata_solve_json_strict_v1(
    request: *const u8,
    request_length: usize,
    max_evaluations: u64,
    result: *mut *mut c_char,
) -> i32 {
    // SAFETY: this function has the same pointer contract as solve_request.
    unsafe {
        solve_request(
            request,
            request_length,
            max_evaluations,
            result,
            decode_strict,
        )
    }
}

unsafe fn solve_request(
    request: *const u8,
    request_length: usize,
    max_evaluations: u64,
    result: *mut *mut c_char,
    decode: fn(&[u8]) -> Result<Problem, i32>,
) -> i32 {
    if result.is_null() || !result.is_aligned() {
        return Status::InvalidArgument as i32;
    }
    // SAFETY: callers provide a writable, aligned, disjoint result slot.
    unsafe { result.write(ptr::null_mut()) };

    if request.is_null() || request_length == 0 || request_length > MAX_REQUEST_BYTES {
        return Status::InvalidArgument as i32;
    }

    // SAFETY: checked the raw length; the caller guarantees readable stable memory.
    let request = unsafe { slice::from_raw_parts(request, request_length) };
    match catch_unwind(|| {
        encode_solution(decode(request)?, max_evaluations).map_err(|status| status as i32)
    }) {
        Ok(Ok(response)) => {
            // SAFETY: the caller's result slot remains valid for the duration of the call.
            unsafe { result.write(response.into_raw()) };
            Status::Ok as i32
        }
        Ok(Err(status)) => status,
        Err(_) => Status::InternalError as i32,
    }
}

/// Releases an output allocated by this library. A null pointer is a no-op.
///
/// # Safety
///
/// The pointer must be null or an unmodified live result from this library
/// instance. It must not be concurrently used or passed to this function twice.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn orchestrata_free_json_v1(result: *mut c_char) {
    if !result.is_null() {
        // SAFETY: the caller returns ownership of an unmodified CString allocation.
        drop(unsafe { CString::from_raw(result) });
    }
}

fn decode_legacy(request: &[u8]) -> Result<Problem, i32> {
    serde_json::from_slice(request).map_err(|_| Status::InvalidProblem as i32)
}

fn decode_strict(request: &[u8]) -> Result<Problem, i32> {
    StrictRequest::decode(request)
        .map(StrictRequest::into_problem)
        .map_err(|status| status as i32)
}

fn encode_solution(problem: Problem, max_evaluations: u64) -> Result<CString, Status> {
    let prepared = PreparedProblem::prepare(problem).map_err(classify_solve_error)?;
    let solution = prepared
        .solve(max_evaluations)
        .map_err(classify_solve_error)?;
    let json =
        serde_json::to_vec(&ResultDocument { solution }).map_err(|_| Status::InternalError)?;
    if json.len() >= MAX_RESULT_BYTES {
        return Err(Status::InternalError);
    }
    CString::new(json).map_err(|_| Status::InternalError)
}

fn classify_solve_error(error: Error) -> Status {
    match error {
        Error::EmptyDomain(_)
        | Error::InvalidVariable(_)
        | Error::InvalidWeight
        | Error::InvalidCountValue { .. }
        | Error::InvalidPrecedence { .. }
        | Error::InvalidNoOverlap { .. }
        | Error::InvalidResourceCapacity(_) => Status::InvalidProblem,
        Error::TooManyAssignments => Status::SearchLimit,
        Error::ScoreOverflow | Error::InvalidSolution => Status::InternalError,
    }
}
