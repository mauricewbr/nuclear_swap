contract;

use ns_lib::abs; // needs to be added

use std::{
    address::*,
    assert::assert,
    block::*,
    chain::auth::*,
    contract_id::ContractId,
    context::{*, call_frames::*},
    hash::*,
    logging::log,
    result::*,
    revert::revert,
    token::*,
    storage::*,
    math::*
};

storage {
    totalSupply: u64,
    lp_token_supply: u64,
}

pub struct RemoveLiquidityReturn {
    eth_amount: u64,
    token_amount: u64,
}

pub struct Logger {
    amount: u64
}

pub struct SenderLog {
    sender: Address
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

//SWAP_FEE: u64,
const SWAP_FEE = 1;

// 2 because of 2 asset pool!
const N = 2;

//uint private constant LIQUIDITY_FEE = (SWAP_FEE * N) / (4 * (N - 1));

//FEE_DENOMINATOR: u64,

// const DECIMALS: u64 = 10**18;

abi NuclearSwap {
    fn get_balance(token: ContractId) -> u64;
    fn get_balances(target: ContractId, asset_id: ContractId) -> u64;
    fn deposit();
    fn withdraw(amount: u64, asset_id: ContractId);
    //fn getVirtualPrice() -> u64;
    fn swap(dx: u64, minDy: u64) -> u64;
    fn add_liquidity(min_liquidity: u64, deadline: u64) -> u64;
    fn remove_liquidity(min_eth: u64, min_tokens: u64, deadline: u64) -> RemoveLiquidityReturn;
    fn test_current_reserve(token_id: b256) -> u64;
}

impl NuclearSwap for Contract {
    fn test_current_reserve(token_id: b256) -> u64 {
        get::<u64>(token_id)
    }

    fn get_balance(token: ContractId) -> u64 {
        let sender = get_msg_sender_address_or_panic();
        let key = key_deposits(sender, token.into());
        get::<u64>(key)
    }

    fn get_balances(target: ContractId, asset_id: ContractId) -> u64 {
        balance_of(target, asset_id)
    }

    fn deposit() {
        assert(msg_asset_id().into() == ETH_ID || msg_asset_id().into() == TOKEN_ID);

        let sender = get_msg_sender_address_or_panic();

        let key = key_deposits(sender, msg_asset_id().into());
        let total_amount = get::<u64>(key) + msg_amount();

        log(SenderLog {sender: sender});
        log(msg_asset_id());

        log(Logger{
            amount: total_amount
        });

        store(key, total_amount);
    }

    fn withdraw(amount: u64, asset_id: ContractId) {
        assert(asset_id.into() == ETH_ID || asset_id.into() == TOKEN_ID);

        let sender = get_msg_sender_address_or_panic();

        // Getting the specific token balance for a specific sender
        let key = key_deposits(sender, asset_id.into());
        let deposited_amount = get::<u64>(key);
        assert(deposited_amount >= amount);

        let new_amount = deposited_amount - amount;
        store(key, new_amount);

        transfer_to_output(amount, asset_id, sender)
    }

    /*
    fn getVirtualPrice() -> u64 {
        let xp: [u64; 2] = [storage.xpX, storage.xpY];
        let d: u64 = _getD(xp);
        let _totalSupply: u64 = storage.totalSupply;
        if _totalSupply > 0 {
            (d * exp(10, 18)) / _totalSupply
        } else {
            0
        }
    }
    */

    fn swap(dx: u64, minDy: u64) -> u64 {
        // assert(i != j);
        let i = 0;
        let j = 1;
        // remove i and j 

        assert(msg_asset_id().into() == ETH_ID || msg_asset_id().into() == TOKEN_ID);

        let asset_id = msg_asset_id().into();
        let forwarded_amount = msg_amount();
        let sender = get_msg_sender_address_or_panic();

        // TO DO: IERC20(tokens[i]).transferFrom(msg.sender, address(this), dx);

        // Getting current reserves of both tokens
        let current_reserve_x = get_current_reserve(ETH_ID);
        let current_reserve_y = get_current_reserve(TOKEN_ID);

        log(Logger{amount: current_reserve_x});
        log(Logger{amount: current_reserve_y});

        // Get new token_in amount:
        assert(dx >= 0);
        let new_reserve_x = current_reserve_x + dx;

        // Get current balances and store in xp:
        let xp: [u64; 2] = [current_reserve_x, current_reserve_y];

        // Computing new token_out amount:
        let new_reserve_y: u64 = _getY(i, j, new_reserve_x, xp);

        // Computing delta token_out:
        // y0 must be >= y1, since x has increased
        // -1 to round down
        let mut dy: u64 = (current_reserve_y - new_reserve_y - 1);

        // Subtract fee from dy
        //let fee: u64 = (dy * SWAP_FEE) / FEE_DENOMINATOR;
        //dy = dy - fee;
        assert(dy >= minDy);

        add_reserve(ETH_ID, dx);
        remove_reserve(TOKEN_ID, dy);
        // TO DO: IERC20(tokens[j]).transfer(msg.sender, dy);

        // Getting new reserves of both tokens
        let new_reserve_x = get_current_reserve(ETH_ID);
        let new_reserve_y = get_current_reserve(TOKEN_ID);

        log(Logger{amount: new_reserve_x});
        log(Logger{amount: new_reserve_y});
        log(Logger{amount: dy});

        dy
    }

    fn add_liquidity(min_liquidity: u64, deadline: u64) -> u64 {
        assert(msg_amount() == 0);
        assert(deadline > height());
        assert(msg_asset_id().into() == ETH_ID || msg_asset_id().into() == TOKEN_ID);

        let FEE_DENOMINATOR = exp(10,6);
        let LIQUIDITY_FEE = (SWAP_FEE * N) / (5 * (N - 1));

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

            let current_eth_reserve = get_current_reserve(ETH_ID);
            let current_token_reserve = get_current_reserve(TOKEN_ID);

            let token_amount = (current_eth_amount * current_token_reserve) / current_eth_reserve;
            // let liquidity_minted = (current_eth_amount * total_liquidity) / current_eth_reserve;

            // Get current balances and store in xp:
            let current_reserves: [u64; 2] = [current_eth_reserve, current_token_reserve];

            // Calculating D, sum of balances in a perfectly balanced pool
            let current_d = _getD(current_reserves);

            // if token ratio is correct, proceed with adding liquidity
            // if token ratio is incorrect, return user balances to contract
            if (current_token_amount >= token_amount) {
                // Adding new tokens to reserves:
                add_reserve(TOKEN_ID, token_amount);
                add_reserve(ETH_ID, current_eth_amount);

                // Calculating ideal LP token amount to mint and send:
                let new_eth_reserve = get_current_reserve(ETH_ID);
                let new_token_reserve = get_current_reserve(TOKEN_ID);
                let new_reserves: [u64; 2] = [new_eth_reserve, new_token_reserve];

                let new_d = _getD(new_reserves); // Calculating D, sum of balances in a perfectly balanced pool

                let idealBalance_eth: u64 = (current_eth_reserve * new_d) / current_d;
                let diff_eth: u64 = abs(new_eth_reserve, idealBalance_eth);
                let net_new_eth_reserve: u64 = (LIQUIDITY_FEE * diff_eth) / FEE_DENOMINATOR;

                let idealBalance_token: u64 = (current_token_reserve * new_d) / current_d;
                let diff_token: u64 = abs(new_token_reserve, idealBalance_token);
                let net_new_token_reserve: u64 = (LIQUIDITY_FEE * diff_token) / FEE_DENOMINATOR;

                let net_new_reserves: [u64; 2] = [net_new_eth_reserve, net_new_token_reserve];
                let net_new_d = _getD(net_new_reserves);

                let liquidity_to_mint = ((net_new_d - current_d) * total_liquidity) / current_d;

                assert(liquidity_to_mint >= min_liquidity);

                // Minting LP tokens and transferring to sender:
                mint(liquidity_to_mint);
                storage.lp_token_supply = total_liquidity + liquidity_to_mint;

                transfer_to_output(liquidity_to_mint, contract_id(), sender);

                // In case user sent more than correct ratio, deposit back extra tokens to contract
                let token_extra = current_token_amount - token_amount;
                if (token_extra > 0) {
                    transfer_to_output(token_extra, ~ContractId::from(TOKEN_ID), sender);
                }
                minted = liquidity_to_mint;
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

            log(Logger{amount: get_current_reserve(ETH_ID)});
            log(Logger{amount: get_current_reserve(TOKEN_ID)});
        }

        // Clear user contract balances after finishing add / create liquidity
        store(token_amount_key, 0);
        store(eth_amount_key, 0);

        minted
    }

    fn remove_liquidity(min_eth: u64, min_tokens: u64, deadline: u64) -> RemoveLiquidityReturn {
        assert(msg_amount() > 0);
        assert(msg_asset_id().into() == (contract_id()).into());
        assert(deadline > height());
        assert(min_eth > 0 && min_tokens > 0);

        let sender = get_msg_sender_address_or_panic();

        let total_liquidity = storage.lp_token_supply;
        assert(total_liquidity > 0);

        let eth_reserve = get_current_reserve(ETH_ID);
        let token_reserve = get_current_reserve(TOKEN_ID);
        let eth_amount = (msg_amount() * eth_reserve) / total_liquidity;
        let token_amount = (msg_amount() * token_reserve) / total_liquidity;

        assert((eth_amount >= min_eth) && (token_amount >= min_tokens));

        burn(msg_amount());
        storage.lp_token_supply = total_liquidity - msg_amount();

        // Remove funds from the reserve
        remove_reserve(TOKEN_ID, token_amount);
        remove_reserve(ETH_ID, eth_amount);

        // Send tokens back
        transfer_to_output(eth_amount, ~ContractId::from(ETH_ID), sender);
        transfer_to_output(token_amount, ~ContractId::from(TOKEN_ID), sender);

        RemoveLiquidityReturn {
            eth_amount: eth_amount,
            token_amount: token_amount,
        }
    }
}

fn exp(base: u64, exponent: u64) -> u64 {
    asm(r1, r2: base, r3: exponent) {
        exp r1 r2 r3;
        r1: u64
    }
}

// XXX -> _mint and _burn can be removed
fn _mint(amount: u64, recipient: Address) {
    mint_to_address(amount, recipient);
}

fn _burn(amount: u64) {
    burn(amount);
}

fn _getYD(i: u64, xp: [u64; 2], d: u64) -> u64 {
    // XXX -> N = 2
    let N = 2;
    // let N: u64 = storage.N;

    let mut s: u64 = 0;
    let mut c: u64 = d;
    let A: u64 = (1000 * (exp(N, N - 1)));
    // following A needs to be replaced by commented A
    // let A: u64 = (1000 * N);
    let a: u64 = A * N;

    let mut _x: u64 = 0;
    let mut counter_i: u64 = 0;
    while counter_i < N {
        if counter_i != i {
            _x = xp[counter_i];
            s = s + _x;
            c = (c * d) / (N * _x);
        }
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
        if abs(y, y_prev) <= 1 {
            break_early = true;
        };
    }
    y
}

fn _getY(i: u64, j: u64, x: u64, xp: [u64; 2]) -> u64 {
    // let A: u64 = (1000 * (N**(N-1)));
    // following A needs to be replaced by commented A
    // XXX -> N = 2 should be dynamic
    let N: u64 = 2;
    let A: u64 = (1000 * (exp(N, N - 1)));
    let a: u64 = A * N;
    let d: u64 = _getD(xp);
    // uint s;
    let mut c: u64 = d;
    let mut s: u64 = 0;
    let mut _x: u64 = 0;
    let mut counter_i: u64 = 0;
    // XXX -> remove while
    while counter_i < N {
        if counter_i == i {
            _x = x;
            s = s + _x;
            c = (c * d) / (N * _x);
        } else if counter_i == j {
            // continue; DO NOTHING HERE, THIS IF CAN BE REMOVED
        } else {
            let _x = xp[counter_i];
            s = s + _x;
            c = (c * d) / (N * _x);
        };

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
        if abs(y, y_prev) <= 1 {
            break_early = true;
        };
    }
    y // revert("y didn't converge");
}

fn _getD(xp: [u64; 2]) -> u64 {
    // N: Number of tokens
    // A: Amplification coefficient multiplied by N^(N-1)
    let current_reserve_x = get_current_reserve(ETH_ID);
    let current_reserve_y = get_current_reserve(TOKEN_ID);

    // XXX -> N = 2
    let N: u64 = 2;

    let A: u64 = (1000 * (exp(N, N - 1)));
    let a: u64 = A * N;
    let mut i = 0;
    let xp: [u64; 2] = [current_reserve_x, current_reserve_y];
    let mut s: u64 = xp[0] + xp[1];
    /*
    while i < N {
        s = s + xp[i];
        i = i + 1;
    }
    */

    let mut d: u64 = s;
    let mut i = 0;
    let mut j = 0;
    let mut break_early = false;
    while i < 255 && break_early == false {
        let mut p: u64 = d;
        while j < N {
            p = (p * d) / (N * xp[j]);
            j = j + 1;
        }
        let d_prev: u64 = d;
        d = ((a * s + N * p) * d) / ((a - 1) * d + (N + 1) * p);

        if abs(d, d_prev) <= 1 {
            break_early = true;
        }
        i = i + 1;
    }
    d // Revert("D didn't converge");
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

fn remove_reserve(token_id: b256, amount: u64) {
    let value = get::<u64>(token_id);
    store(token_id, value - amount);
}
