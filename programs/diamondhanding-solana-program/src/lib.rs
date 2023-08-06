use anchor_lang::{prelude::*, solana_program::clock, system_program};
use std::convert::TryInto;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("5Zm2UQMSM63NLJGkQYP6xqqGm2EPzYyVNtyPpJnJb5iD");

#[program]
mod diamondhanding_solana_program {
    use super::*;

    pub fn init_sol_store(
        ctx: Context<InitSolStore>,
        unlock_date: i64,
        can_manually_unlock: bool,
    ) -> Result<()> {
        let sol_store = &mut ctx.accounts.sol_store;
        sol_store.unlock_date = unlock_date.clone();
        sol_store.can_manually_unlock = can_manually_unlock.clone();
        sol_store.signer = ctx.accounts.signer.key.clone();

        msg!(
            "Initialized new sol store. Unlock date: {}, Can manually unlock: {}, Signer: {}",
            sol_store.unlock_date,
            sol_store.can_manually_unlock,
            sol_store.signer
        );
        Ok(())
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        let sol_store = &mut ctx.accounts.sol_store;
        let signer = &ctx.accounts.signer;
        let system_program: &Program<'_, System> = &ctx.accounts.system_program;

        let cpi_context = CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: signer.to_account_info(),
                to: sol_store.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;
        msg!("Deposited {} lamports to {}", amount, sol_store.signer);
        Ok(())
    }

    pub fn withdraw_sol_and_close_account(ctx: Context<WithdrawSolAndCloseAccount>) -> Result<()> {
        let signer = &ctx.accounts.signer;

        msg!(
            "Account closed, funds sent back to {}",
            signer.clone().key()
        );
        Ok(())
    }
}

#[account]
pub struct SolStore {
    // this means that the Account will be known as a Counter and it will store a count data for us.
    // This is a bit like a schema for a MongoDB model. 1 Account = 1 document.
    // Naturally the public key is the document id already.
    unlock_date: i64,
    can_manually_unlock: bool,
    signer: Pubkey,
}

impl SolStore {
    pub fn is_unlocked(&self) -> bool {
        fn now_ts() -> Result<i64> {
            Ok(clock::Clock::get()?.unix_timestamp.try_into().unwrap())
        }
        self.unlock_date <= now_ts().unwrap() || self.can_manually_unlock
    }

    pub const MAX_SIZE: usize = 8 + (1) + (32);
}

#[derive(Accounts)]
pub struct InitSolStore<'info> {
    // this is a bit like defining the function types and constraints before implementing it.
    #[account(init, seeds = [b"sol", signer.key().as_ref()], bump, payer = signer, space = 8 + SolStore::MAX_SIZE)]
    pub sol_store: Account<'info, SolStore>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut, has_one = signer)]
    pub sol_store: Account<'info, SolStore>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolAndCloseAccount<'info> {
    #[account(mut, has_one = signer, close = signer, constraint = sol_store.is_unlocked() @ MyError::LockedSolStore)]
    pub sol_store: Account<'info, SolStore>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[error_code]
pub enum MyError {
    #[msg("Sol Store is LOCKED")]
    LockedSolStore,
}
