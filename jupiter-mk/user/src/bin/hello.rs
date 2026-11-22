//! A minimal user program used to prove IPC and spawning.

#![no_std]
#![no_main]

use jupiter_user::*;

fn main() {
    print("hello from user space!\n");
}

jupiter_user::entry!(main);
