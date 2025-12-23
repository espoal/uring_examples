extern crate liburing_rs;

use std::{mem::zeroed, ptr};

use liburing_rs::*;

fn main() {

    println!("Hello, world!");

    let mut ring = unsafe { zeroed::<io_uring>() };
    let ring = &raw mut ring;
    let r = unsafe { io_uring_queue_init(64, ring, 0) };
    assert_eq!(r, 0);

    println!("Ring successfully initialized!");

    // Grab a pointer to an unused SQE from the SQ
    let sqe = unsafe { io_uring_get_sqe(ring) };
    assert!(!sqe.is_null());

    println!("SQE successfully retrieved!");

    unsafe { io_uring_prep_nop(sqe); }
    unsafe { io_uring_sqe_set_data64(sqe, 0xf9f1); }

    println!("SQE successfully prepared!");



    // Submit the SQ to the kernel for processing. `io_uring_submit` returns
    // the number of submitted entries that are going to be processed. In this case,
    // we only have 1 work item.
    let n = unsafe { io_uring_submit(ring) };
    assert_eq!(n, 1);
    println!("SQE successfully submitted!");

    // Grab the first CQE off the queue and make sure it has the same `user_data`
    // field that we originally set.
    //
    let mut cqe = ptr::null_mut::<io_uring_cqe>();
    unsafe { io_uring_wait_cqe(ring, &raw mut cqe) };
    println!("CQE successfully waited for!");

    assert!(!cqe.is_null());
    assert_eq!(unsafe { (*cqe).user_data }, 0xf9f1);
    println!("CQE successfully verified!");

    // Mark the CQE as seen so that its spot can be reused as more
    // completions arrive.
    //
    unsafe { io_uring_cqe_seen(ring, cqe) };
    println!("CQE successfully marked as seen!");

    // Tear down the ring.
    unsafe { io_uring_queue_exit(ring) };
    println!("Ring successfully exited!");

}

