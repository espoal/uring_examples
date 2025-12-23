extern crate liburing_rs;

use std::{mem::zeroed, ptr, time::Duration};

use liburing_rs::*;

fn main() {

    println!("Hello, world!");

    let mut ring = unsafe { zeroed::<io_uring>() };
    let ring = &raw mut ring;
    let r = unsafe { io_uring_queue_init(64, ring, 0) };
    //assert_eq!(r, 0);

    println!("Ring successfully initialized!");

}

