use std::{ffi::CString, slice};

use orchestrata_solver::capacity_matching::{CapacityMatchProblem, match_capacity};

use crate::adapter::{CAPACITY_REQUEST_LIMIT, pack_result};

/// Return the largest feasible assignment or an error. No partial result is returned.
///
/// # Safety
/// A nonnull request must be readable for len bytes and disjoint from any mutable use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn demo_match_capacity(request: *const u8, len: usize) -> i64 {
    if request.is_null() || !(1..=CAPACITY_REQUEST_LIMIT).contains(&len) {
        return -7;
    }
    // SAFETY: request ownership stays with the caller for the duration of this call.
    let request = unsafe { slice::from_raw_parts(request, len) };
    // The wire contract admits named fields only, not Serde's positional struct form.
    if request.iter().copied().find(|b| !b.is_ascii_whitespace()) != Some(b'{') {
        return -6;
    }
    let Ok(problem) = serde_json::from_slice::<CapacityMatchProblem>(request) else {
        return -6;
    };
    let Ok(assignment) = match_capacity(&problem) else {
        return -6;
    };
    let Ok(json) = serde_json::to_vec(&assignment) else {
        return -3;
    };
    let Ok(result) = CString::new(json) else {
        return -3;
    };
    let len = result.as_bytes().len() as u32;
    pack_result(result.into_raw(), len)
}
