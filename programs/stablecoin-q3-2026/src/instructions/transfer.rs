use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions,
        },
        state::Mint as MintState,
    },
    token_interface::{transfer_checked_with_fee, TokenInterface, TransferCheckedWithFee},
};

#[derive(Accounts)]
pub struct TransferWithFees<'info> {
    pub authority: Signer<'info>,

    #[account(mut)]
    /// CHECK : token accounts and will be validated by token_program
    pub from: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK : token accounts and will be validated by token_program
    pub to: UncheckedAccount<'info>,

    /// CHECK : mint account
    pub mint: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TransferWithFees<'info> {
    pub fn transfer(&self, amount: u64, decimals: u8) -> Result<()> {
        let data = self.mint.try_borrow_data()?;
        let mint = StateWithExtensions::<MintState>::unpack(&data)?;
        let fee = mint
            .get_extension::<TransferFeeConfig>()?
            .calculate_epoch_fee(Clock::get()?.epoch, amount)
            .ok_or(ProgramError::InvalidArgument)?;
        transfer_checked_with_fee(
            CpiContext::new(
                self.token_program.key(),
                TransferCheckedWithFee {
                    token_program_id: self.token_program.to_account_info(),
                    source: self.from.to_account_info(),
                    mint: self.mint.to_account_info(),
                    destination: self.to.to_account_info(),
                    authority: self.authority.to_account_info(),
                },
            ),
            amount,
            decimals,
            fee,
        )
    }
}
