contract;

use ns_lib::abs; // needs to be added
use std::address::*;
use std::assert::assert;
use std::block::*;
use std::chain::auth::*;
use std::contract_id::ContractId;
use std::context::{*, call_frames::*};
use std::hash::*;
use std::result::*;
use std::revert::revert;
use std::token::*;
use std::storage::*;
use std::math::*;

storage {
    totalSupply: u64,
    lp_token_supply: u64
}

// Token ID of Ether
const ETH_ID = 0x0000000000000000000000000000000000000000000000000000000000000000;

// Contract ID of the token on the other side of the pool.
// Modify at compile time for different pool.
const TOKEN_ID = 0xb72c566e5a9f69c98298a04d70a38cb32baca4d9b280da8590e0314fb00c59e0;

// Storage delimited
const S_DEPOSITS: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

/// Minimum ETH liquidity to open a pool.
const MINIMUM_LIQUIDITY = 1; //A more realistic value would be 1000000000;

// const DECIMALS: u64 = 10**18;

abi NuclearSwap {
    // fn _mint(amount: u64, recipient: Address); // same as mint_to_address: 
    // fn _burn(amount: u64); // misses from: ContractId???
    // fn _xp(N: u64, xp: [u64;2], balances: [u64; 2], multipliers: [u64; 2]) -> [u64; 2]; // Missing return array
    // fn _getD(N: u64, A: u64, xp: [u64; 2]) -> u64; // N = 2
    // fn _getY(N: u64, i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64; // N = 2
    fn getVirtualPrice(something: u64) -> u64;
    //fn swap(i: u64, j: u64, dx: u64, minDy: u64) -> u64;
    fn add_liquidity(min_liquidity: u64, deadline: u64) -> u64;
}

impl NuclearSwap for Contract {
    fn getVirtualPrice(something: u64)-> u64 {
        let d: u64 = 10;//_getD(_xp());
        // let _totalSupply: u64 = storage.totalSupply;
        // if  _totalSupply > 0 {
        //     (d*10^DECIMALS) / _totalSupply
        // } else {
        //     0
        // }
        // Exponentiate.flow(2,4)
        d
    }

    fn add_liquidity(min_liquidity: u64, deadline: u64) -> u64 {
        assert(msg_amount() == 0);
        assert(deadline > height());
        assert(msg_asset_id().into() == ETH_ID || msg_asset_id().into() == TOKEN_ID);

        let sender = get_msg_sender_address_or_panic();
        let total_liquidity = storage.lp_token_supply;

        let eth_amount_key = key_deposits(sender, ETH_ID);
        let current_eth_amount = get::<u64>(eth_amount_key);
        
        let token_amount_key = key_deposits(sender, TOKEN_ID);
        let current_token_amount = get::<u64>(token_amount_key);

        assert(current_eth_amount > 0);

        let mut minted: u64 = 0;
        if total_liquidity > 0 {
            assert(min_liquidity > 0);

            let eth_reserve = get_current_reserve(ETH_ID);
            let token_reserve = get_current_reserve(TOKEN_ID);
            let token_amount = (current_eth_amount * token_reserve) / eth_reserve;
            let liquidity_minted = (current_eth_amount * total_liquidity) / eth_reserve;

            assert(liquidity_minted >= min_liquidity);

            // if token ratio is correct, proceed with adding liquidity
            // if token ratio is incorrect, return user balances to contract
            if(current_token_amount >= token_amount) {
                add_reserve(TOKEN_ID, token_amount);
                add_reserve(ETH_ID, current_eth_amount);

                mint(liquidity_minted);
                storage.lp_token_supply = total_liquidity + liquidity_minted;

                transfer_to_output(liquidity_minted, contract_id(), sender);

                // In case user sent more than correct ratio, deposit back extra tokens to contract
                let token_extra = current_token_amount - token_amount;
                if (token_extra > 0) {
                    transfer_to_output(token_extra, ~ContractId::from(TOKEN_ID), sender);
                }
                minted = liquidity_minted;
            } else {
                transfer_to_output(current_token_amount, ~ContractId::from(TOKEN_ID), sender);
                transfer_to_output(current_eth_amount, ~ContractId::from(ETH_ID), sender);
                minted = 0;
            }
        } else {
            assert(current_eth_amount > MINIMUM_LIQUIDITY);

            let initial_liquidity = current_eth_amount;

            // Add funds to the reserve
            add_reserve(TOKEN_ID, current_token_amount);
            add_reserve(ETH_ID, current_eth_amount);

            // Mint the LP token
            mint(initial_liquidity);
            storage.lp_token_supply = initial_liquidity;
            
            // Transfering LP token to user balance
            transfer_to_output(initial_liquidity, contract_id(), sender);
            minted = initial_liquidity;
        }

        // Clear user contract balances after finishing add / create liquidity
        store(token_amount_key, 0);
        store(eth_amount_key, 0);

        minted
    }
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
    let A: u64 = (1000 * exp(N, (N-1)));
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
    let A: u64 = (1000 * exp(N, (N-1)));
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
    let A: u64 = (1000 * exp(N, (N-1)));
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

// Return the sender as an Address or panic
// XXX -> Put in library
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

// Compute the storage slot for an address's deposits.
// XXX -> Put in library
fn key_deposits(a: Address, asset_id: b256) -> b256 {
    let inner = sha256((a.into(), asset_id));
    sha256((S_DEPOSITS, inner))
}

// Return token reserve balance
// XXX -> Put in library
fn get_current_reserve(token_id: b256) -> u64 {
    get::<u64>(token_id)
}

// Add amount to the token reserve
// XXX -> Put in library
fn add_reserve(token_id: b256, amount: u64) {
    let value = get::<u64>(token_id);
    store(token_id, value + amount);
}