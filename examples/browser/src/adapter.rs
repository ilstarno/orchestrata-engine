use std::{
    alloc::{Layout, alloc_zeroed, dealloc},
    ffi::{CStr, c_char},
    ptr,
};

const REQUEST_LIMIT: usize = 128 * 1024;
pub(super) const CAPACITY_REQUEST_LIMIT: usize = 256 * 1024;
const EVALUATION_LIMIT: u32 = 4096;

#[unsafe(no_mangle)]
pub extern "C" fn demo_alloc(len: usize) -> *mut u8 {
    if !(1..=CAPACITY_REQUEST_LIMIT).contains(&len) {
        return ptr::null_mut();
    }
    // SAFETY: bounded nonzero length gives a valid allocation layout.
    unsafe { alloc_zeroed(request_layout(len)) }
}

/// # Safety
/// The caller must pass exactly one live allocation and its original length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn demo_free(request: *mut u8, len: usize) {
    if !request.is_null() {
        // SAFETY: the caller supplies the allocation and its exact length.
        unsafe { dealloc(request, request_layout(len)) };
    }
}

fn request_layout(len: usize) -> Layout {
    Layout::array::<u8>(len).expect("bounded request length")
}

/// Returns the packed result pointer and length, or a negative status code.
/// A nonnegative result owns the pointer in its high 32 bits.
///
/// # Safety
/// The request must be readable for len bytes and disjoint from any mutable use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn demo_solve(request: *const u8, len: usize, budget: u32) -> i64 {
    if request.is_null() || !(1..=REQUEST_LIMIT).contains(&len) {
        return -4;
    }
    if !(1..=EVALUATION_LIMIT).contains(&budget) {
        return -5;
    }
    let mut result = ptr::null_mut();
    // SAFETY: the caller supplies readable request memory, and the result slot
    // is distinct on this function's stack.
    let status = unsafe {
        orchestrata_ffi::orchestrata_solve_json_v1(request, len, u64::from(budget), &mut result)
    };
    if status != 0 {
        return -i64::from(status);
    }
    // SAFETY: a successful response from the engine is a live C string.
    let len = unsafe { CStr::from_ptr(result) }.to_bytes().len() as u32;
    pack_result(result, len)
}

pub(super) fn pack_result(result: *mut c_char, len: u32) -> i64 {
    i64::from(result as u32) << 32 | i64::from(len)
}

/// # Safety
/// The caller must pass exactly one live result pointer received from demo_solve.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn demo_free_result(result: *mut c_char) {
    // SAFETY: return ownership to the allocator that produced this exact pointer.
    unsafe { orchestrata_ffi::orchestrata_free_json_v1(result) };
}
