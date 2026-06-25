use anchor_lang::prelude::*;

declare_id!("Dvc6oWvACkeBdEo1hbk8DTxSk8twWGTgXWVsfHtdMwPs");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.balance = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Update>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.balance = vault
            .balance
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Update>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // PLANTED BUG: no `require!(amount <= vault.balance)` and a wrapping
        // subtraction. A withdraw larger than the balance wraps to a huge u64
        // instead of failing, so the balance INCREASES on a withdraw. The
        // correct version (require! + checked_sub) is shown in the README.
        vault.balance = vault.balance.wrapping_sub(amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Vault::INIT_SPACE,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(
        mut,
        seeds = [b"vault", authority.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub authority: Pubkey,
    pub balance: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("arithmetic overflow")]
    Overflow,
    #[msg("insufficient funds")]
    InsufficientFunds,
}
