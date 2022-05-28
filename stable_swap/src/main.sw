contract;

use ns_lib::abs; // needs to be added
use std::address::Address;
use std::token::{mint_to_address, burn};

abi NuclearSwap {
    fn _mint(amount: u64, recipient: Address); // same as mint_to_address: 
    fn _burn(amount: u64); // misses from: ContractId???
    fn _xp(N: u64, xp: [u64;2], balances: [u64; 2], multipliers: [u64; 2]) -> [u64; 2]; // Missing return array
    fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64; // N = 2
    fn _getY(i: u64, j: u64, x: u64, N: [u64; 2]) -> u64; // N = 2
}

impl NuclearSwap for Contract {
    fn _mint(amount: u64, recipient: Address) {
        mint_to_address(amount, recipient);
    }

    fn _burn(amount: u64) {
        burn(amount);
    }

    fn _xp(N: u64, xp: [u64; 2], balances: [u64; 2], multipliers: [u64; 2]) {
        let mut counter = 0;
        while counter < N { // N = 2
            // needs fix -> static array
            xp[counter] = balances[counter] * multipliers[counter];
            counter = counter + 1;
        }
        // return xp ??? How to return array?
    }

    fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64 {
        // N: Number of tokens
        // A: Amplification coefficient multiplied by N^(N-1)
        let a: u64 = A * N;
        let mut counter = 0;
        let mut s: u64 = xp[0];
        while counter < N {
            s = s + xp[counter];
            counter = counter + 1;
        }

        let mut d: u64 = s;
        let mut counter_i = 0;
        let mut counter_j = 0;
        while counter_i < 255 {
            let mut p: u64 = d;
            while counter_j < N {
                p = (p * d) / (N * xp[counter_j]);
                counter_j = counter_j + 1;
            }
            let d_prev: u64 = d;
            d = ((a * s + N * p) * d) / ((a - 1) * d + (N + 1) * p);

            if abs(d, d_prev) <= 1 {
                return d
            }
            counter_i = counter_i + 1;
        }
        // Revert("D didn't converge");
    }

    fn _getY(i: u64, j: u64, x: u64, N: [u64; 2]) -> u64 {
        
    }
}