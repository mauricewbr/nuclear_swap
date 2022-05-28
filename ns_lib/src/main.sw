library ns_lib;

use std::{storage::*};

pub fn abs(x: u64, y: u64) -> u64 {
    let mut res: u64 = 0;
    if x >= y {
        res = x - y;
    } else {
        res = y - x;
    }
    res
}

pub fn get_b256(key: b256) -> b256 {
    asm(r1: key, r2) {
        move r2 sp;
        cfei i32;
        srwq r2 r1;
        r2: b256
    }
}

// Store b256 values on memory
pub fn store_b256(key: b256, value: b256) {
    asm(r1: key, r2: value) {
        swwq r1 r2;
    };
}
