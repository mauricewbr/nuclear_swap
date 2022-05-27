contract;

use swapMath::abs;
use std::token::{mint_to_address, burn};

abi NuclearSwap {
    fn _mint(amount: u64, recipient: Address); // same as mint_to_address: 
    fn _burn(amount: u64); // misses from: ContractId???
    fn _xp() -> u64; // misses array structure
    fn _getD(xp: u64) -> u64; // misses xp return array structure

}

impl NuclearSwap for Contract {
    fn _mint(amount: u64, recipient: Address) {
        mint_to_address(amount, recipient);
    }

    fn _burn(amount: u64) {
        burn(amount);
    }

    fn _xp() {
        while counter < ??? { // missing
            // xp[i] = balances[i] * multipliers[i];
            counter = counter + 1;
        }
    }

    fn _getD(xp: u64) -> u64 {
        // N: Number of tokens
        // A: Amplification coefficient multiplied by N^(N-1)
        let a: u64 = A * N;
        let s: u64;
        while counter < ??? { // missing
            // s = s + xp[i]
            counter = counter + 1;
        }

        let d: u64 = s;
        while counter_i < 255 {
            let p: u64 = d;
            while counter_j < N {
                p = (p * d) / (N * xp[j]);
                counter_j = counter_j + 1;
            }
            let d_prev: u64 = d;
            d = ((a * s + N * p) * d) / ((a - 1) * d + (N + 1) * p);

            if abs(d, d_prev) <= 1 {
                d
            }
            counter_i = counter_i + 1;
        }
        // Revert("D didn't converge");
    }
}

/* XXX -> Needs to be included in library
fn mathAbs(x: u64, y: u64) -> u64 {
    if x >= y {
        x - y
    } else {
        y - x
    }
}
*/