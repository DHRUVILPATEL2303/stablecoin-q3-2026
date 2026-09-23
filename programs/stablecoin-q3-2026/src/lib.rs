pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("GyS3h3kXEHADYhzHXR2r9VBMYejFNeGwg7Ln66v2bqo5");

#[program]
pub mod stablecoin_q3_2026 {
    use super::*;

    pub fn initalize(ctx: Context<InitalizeMint>) -> Result<()> {
        ctx.accounts.initialize()
    }

    pub fn transfer(ctx: Context<TransferWithFees>, amount: u64, decimals: u8) -> Result<()> {
        ctx.accounts.transfer(amount, decimals)
    }

    pub fn unfreeze(ctx: Context<UnfreezeKycAccount>) -> Result<()> {
        ctx.accounts.unfreeze()
    }
}
