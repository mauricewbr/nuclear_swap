contract;

use ns_lib::abs; // needs to be added
use std::address::Address;
use std::token::{mint_to_address, burn};

abi NuclearSwap {
    // fn _mint(amount: u64, recipient: Address); // same as mint_to_address: 
    // fn _burn(amount: u64); // misses from: ContractId???
    // fn _xp(N: u64, xp: [u64;2], balances: [u64; 2], multipliers: [u64; 2]) -> [u64; 2]; // Missing return array
    // fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64; // N = 2
    // fn _getY(N: u64, i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64; // N = 2
}

impl NuclearSwap for Contract {
    
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

fn _getY(N: u64, i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64 {
    let a: u64 = A * N;
    //let xp: [u64; 2] = [1; 1000000000000];
    let d: u64 = _getD(xp);
    // uint s;
    let c: u64 = d;
    let mut counter: u64 = 0;
    while k < N {
        if k == i {
            let _x: u64 = x;
        }
    }
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
    while i < 255 {
        let mut p: u64 = d;
        while j < N {
            p = (p * d) / (N * xp[j]);
            j = j + 1;
        }
        let d_prev: u64 = d;
        d = ((a * s + N * p) * d) / ((a - 1) * d + (N + 1) * p);

        if abs(d, d_prev) <= 1 {
            return d
        }
        i = i + 1;
    }
    // Revert("D didn't converge");
}