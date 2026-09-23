use anchor_lang::prelude::*;

#[constant]
pub const COUNTER_SEED: &[u8] = b"counter";

#[constant]
pub const HELLO_WORLD_LAMPORTS: u64 = 1;

#[constant]
pub const MAX_COUNT: u64 = 10;

#[constant]
pub const DECIMALS: u8 = 6;

#[constant]
pub const TRANSFER_FEE_BPS: u16 = 100;

#[constant]
pub const MAXIMUM_FEE: u64 = u64::MAX;

#[constant]
pub const TOKEN_NAME: &str = "DHRUVIL USD";

#[constant]
pub const TOKEN_SYMBOL: &str = "DUSD";

#[constant]
pub const TOKEN_URI: &str = "https://dhruvilxsol.me/dusd.json";
