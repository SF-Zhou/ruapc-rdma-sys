//! FFI wrapper functions for RDMA verbs
//!
//! These wrappers provide inline optimizations over raw libibverbs
//! function pointers accessed through ops vtable.

use crate::{ibv_cq, ibv_qp, ibv_recv_wr, ibv_send_wr, ibv_wc};
use std::os::raw::c_int;

/// Requests notification for completion queue events
///
/// This is an inline wrapper that calls through the context's ops vtable
/// for better performance than calling the global function.
#[inline(always)]
pub unsafe fn ibv_req_notify_cq(cq: *mut ibv_cq, solicited_only: c_int) -> c_int {
    unsafe { (*(*cq).context).ops.req_notify_cq.unwrap_unchecked()(cq, solicited_only) }
}

/// Polls completion queue for work completions
///
/// Returns the number of completions polled (negative on error)
#[inline(always)]
pub unsafe fn ibv_poll_cq(cq: *mut ibv_cq, num_entries: c_int, wc: *mut ibv_wc) -> c_int {
    unsafe { (*(*cq).context).ops.poll_cq.unwrap_unchecked()(cq, num_entries, wc) }
}

/// Posts a send work request to a queue pair
#[inline(always)]
pub unsafe fn ibv_post_send(
    qp: *mut ibv_qp,
    wr: *mut ibv_send_wr,
    bad_wr: *mut *mut ibv_send_wr,
) -> c_int {
    unsafe { (*(*qp).context).ops.post_send.unwrap_unchecked()(qp, wr, bad_wr) }
}

/// Posts receive work request to queue pair
///
/// Returns 0 on success, negative on error, and sets bad_wr
/// to point to the first invalid request if the batch fails.
#[inline(always)]
pub unsafe fn ibv_post_recv(
    qp: *mut ibv_qp,
    wr: *mut ibv_recv_wr,
    bad_wr: *mut *mut ibv_recv_wr,
) -> c_int {
    unsafe { (*(*qp).context).ops.post_recv.unwrap_unchecked()(qp, wr, bad_wr) }
}
