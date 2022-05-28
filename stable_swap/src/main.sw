contract;

use ns_lib::abs; // needs to be added
// use std::address::Address;
// use std::token::{mint_to_address, burn};
// use std::storage::*;
// use std::math::*;

use std::{
    address::*,
    token::{mint_to_address, burn},
    storage::*,
    math::*,
    assert::assert,
    block::*,
    chain::auth::*,
    context::{*, call_frames::*},
    contract_id::ContractId,
    hash::*,
    revert::revert,
    // token::*,
    result::*
};

storage {
    N: u64,
    totalSupply: u64,
    xp: [u64; 2],
    multipliers: [u64; 2]
}

abi NuclearSwap {
    // fn _mint(amount: u64, recipient: Address); // same as mint_to_address: 
    // fn _burn(amount: u64); // misses from: ContractId???
    // fn _xp(N: u64, xp: [u64;2], balances: [u64; 2], multipliers: [u64; 2]) -> [u64; 2]; // Missing return array
    // fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64; // N = 2
    // fn _getY(N: u64, i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64; // N = 2
    fn getVirtualPrice() -> u64;
    fn swap(i: u64, j:u64, dx: u64, minDy: u64) -> u64;
}

impl NuclearSwap for Contract {
    fn getVirtualPrice() -> u64 {
        let d: u64 = 10;//_getD(_xp());
        let _totalSupply: u64 = storage.totalSupply;
        if  _totalSupply > 0 {
            (d*exp(10,18)) / _totalSupply
        } else {
            0
        }
    }

    fn swap(i: u64, j:u64, dx: u64, minDy: u64) -> u64 {
        assert(i != j);
        let asset_id = msg_asset_id().into();
        let fowarded_amount = msg_amount();
        let sender = get_msg_sender_address_or_panic();
        
        // TO DO: IERC20(tokens[i]).transferFrom(msg.sender, address(this), dx);
        let xpi: u64 = storage.xp[i];
        let multi: u64 =  storage.multipliers[i];
        let x: u64 = xpi + dx * multi;
        let y0: u64 = xp[j];
        let y1: u64 = _getY(i, j, x, xp);
        // y0 must be >= y1, since x has increased
        // -1 to round down
        let mut dy: u64 = (y0 - y1 - 1) / storage.multipliers[j];

        // Subtract fee from dy
        let fee: u64 = (dy * SWAP_FEE) / FEE_DENOMINATOR;
        dy -= fee;
        assert(dy >= minDy);

        balances[i] += dx;
        balances[j] -= dy;

        // IERC20(tokens[j]).transfer(msg.sender, dy);

        let dummy: u64 = 1;
        dummy
    }
}

/// Return the sender as an Address or panic
fn get_msg_sender_address_or_panic() -> Address {
    let result: Result<Sender, AuthError> = msg_sender();
    let mut ret = ~Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    if result.is_err() {
        revert(0);
    } else {
        let unwrapped = result.unwrap();
        if let Sender::Address(v) = unwrapped {
            ret = v;
        } else {
            revert(0);
        };
    };

    ret
}

fn exp (base: u64, exponent: u64) -> u64 {
    asm (r1, r2: base, r3: exponent) {
        exp r1 r2 r3;
        r1: u64
    }
}

fn _mint(amount: u64, recipient: Address) {
    mint_to_address(amount, recipient);
}

fn _burn(amount: u64) {
    burn(amount);
}

/*
fn _xp(N: u64, xp: [u64; 2], balances: [u64; 2], multipliers: [u64; 2]) {
    let mut counter = 0;
    while counter < N { // N = 2
        // needs fix -> static array
        xp[counter] = balances[counter] * multipliers[counter];
        counter = counter + 1;
    }
    // return xp ??? How to return array?
}
*/

fn _getYD(N: u64, i: u64, xp: [u64; 2], d: u64) -> u64 {
    let mut _x: u64 = 0;
    let mut s: u64 = 0;
    let mut c: u64 = 0;
    // let A: u64 = (1000 * (N**(N-1)));
    // following A needs to be replaced by commented A
    let A: u64 = (1000 * N);
    let a: u64 = A * N;
    let mut counter_i: u64 = 0;
    while counter_i < N {
        if counter_i != i {
            _x = xp[counter_i];
        }
        s = s + _x;
        c = (c * d) / (N * _x);
        counter_i = counter_i + 1;
    }
    c = (c * d) / (N * a);
    let b: u64 = s + d / a;

    // Newton's method
    let mut y_prev: u64 = 0;
    let mut y: u64 = d;
    let mut counter_j: u64 = 0;
    let mut break_early = false;
    while counter_j < 255 && break_early == false {
        y_prev = y;
        y = (y * y + c) / (2 * y + b - d);
        if abs(y , y_prev) <= 1 {
            break_early = true;
        };
    }
    y
}

fn _getY(N: u64, i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64 {
    // let A: u64 = (1000 * (N**(N-1)));
    // following A needs to be replaced by commented A
    let A: u64 = (1000 * N);
    let a: u64 = A * N;
    let d: u64 = _getD(N, A, xp);
    // uint s;
    let mut c: u64 = d;
    let mut s: u64 = 0;
    let mut _x: u64 = 0;
    let mut counter_i: u64 = 0;
    while counter_i < N{
        if counter_i == i {
            _x = x;
        } else if counter_i == j {
            // continue;
        } else {
            let _x = xp[counter_i];
        };
        s = s + _x;
        c = (c * d) / (N * _x);
        counter_i = counter_i + 1;
    }
    c = (c * d) / (N * a);
    let b: u64 = s + d / a;

    // Newton's method
    let mut y_prev: u64 = 0;
    let mut y: u64 = d;
    let mut counter_j: u64 = 0;
    let mut break_early = false;
    while counter_j < 255 && break_early == false{
        y_prev = y;
        y = (y * y + c) / (2 * y + b - d);
        if abs(y , y_prev) <= 1{
            break_early = true;
        };
    }
    y
    // revert("y didn't converge");
}

fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64 {
    // N: Number of tokens
    // A: Amplification coefficient multiplied by N^(N-1)
    let a: u64 = A * N;
    let mut i = 0;
    //let xp: [u64; 2] = [1; 1000000000000]; 
    let mut s: u64 = xp[0];
    while i < N {
        s = s + xp[i];
        i = i + 1;
    }

    let mut d: u64 = s;
    let mut i = 0;
    let mut j = 0;
    let mut break_early = false;
    while i < 255  && break_early == false{
        let mut p: u64 = d;
        while j < N {
            p = (p * d) / (N * xp[j]);
            j = j + 1;
        }
        let d_prev: u64 = d;
        d = ((a * s + N * p) * d) / ((a - 1) * d + (N + 1) * p);

        if abs(d, d_prev) <= 1{
            break_early = true;
        }
        i = i + 1;
    }
    d
    // Revert("D didn't converge");
}