use crate::error::DiamondHandingError;
use crate::state::store::Store;
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer as SplTransfer},
};
pub mod error;
pub mod state;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("5Zm2UQMSM63NLJGkQYP6xqqGm2EPzYyVNtyPpJnJb5iD");

#[program]
mod diamondhanding_solana_program {
    use anchor_spl::token::CloseAccount;

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
        let sol_store = &mut ctx.accounts.sol_store;
        if sol_store.is_unlocked() {
            Ok(())
        } else {
            Err(DiamondHandingError::LockedStore.into())
        }
    }

    pub fn init_spl_store(
        ctx: Context<InitSplStore>,
        unlock_date: i64,
        can_manually_unlock: bool,
    ) -> Result<()> {
        let spl_store = &mut ctx.accounts.spl_store;
        spl_store.unlock_date = unlock_date.clone();
        spl_store.can_manually_unlock = can_manually_unlock.clone();
        spl_store.signer = ctx.accounts.signer.key.clone();

        msg!(
            "Initialized new SPL store. Unlock date: {}, Can manually unlock: {}, Signer: {}",
            spl_store.unlock_date,
            spl_store.can_manually_unlock,
            spl_store.signer
        );
        Ok(())
    }

    pub fn init_associated_token_account(_ctx: Context<InitAssociatedTokenAccount>) -> Result<()> {
        Ok(())
    }

    pub fn deposit_spl_token(ctx: Context<DepositSplToken>, amount: u64) -> Result<()> {
        let destination = &ctx.accounts.to_ata;
        let source = &ctx.accounts.from_ata;
        let token_program = &ctx.accounts.token_program;
        let authority = &ctx.accounts.signer;

        // Transfer tokens from taker to initializer
        let cpi_accounts = SplTransfer {
            from: source.to_account_info().clone(),
            to: destination.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();

        anchor_spl::token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;
        Ok(())
    }

    pub fn withdraw_spl_and_close_account(ctx: Context<WithdrawSplAndCloseAccounts>) -> Result<()> {
        let spl_store = &mut ctx.accounts.spl_store;
        if spl_store.is_unlocked() {
            let destination = &ctx.accounts.signer_ata;
            let account = &ctx.accounts.store_ata;
            let token_program = &ctx.accounts.token_program;
            let authority = &ctx.accounts.spl_store;
            let amount: u64 = ctx.accounts.store_ata.amount;
            let mint_key = &ctx.accounts.mint.key();
            let signer_key = &ctx.accounts.signer.key();

            let cpi_accounts = SplTransfer {
                from: account.to_account_info().clone(),
                to: destination.to_account_info().clone(),
                authority: authority.to_account_info().clone(),
            };
            let cpi_program = token_program.to_account_info();

            let bump = *ctx.bumps.get("spl_store").unwrap();
            let signer: &[&[&[u8]]] = &[&[b"spl", signer_key.as_ref(), mint_key.as_ref(), &[bump]]];
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(cpi_program.clone(), cpi_accounts, signer),
                amount,
            )?;

            let close_cpi_accounts: CloseAccount<'_> = CloseAccount {
                account: account.to_account_info().clone(),
                destination: ctx.accounts.signer.to_account_info().clone(),
                authority: authority.to_account_info().clone(),
            };

            anchor_spl::token::close_account(CpiContext::new_with_signer(
                cpi_program.clone(),
                close_cpi_accounts,
                signer,
            ))?;

            Ok(())
        } else {
            Err(DiamondHandingError::LockedStore.into())
        }
    }
}

#[derive(Accounts)]
pub struct InitSolStore<'info> {
    // this is a bit like defining the function types and constraints before implementing it.
    #[account(init, seeds = [b"sol", signer.key().as_ref()], bump, payer = signer, space = 8 + Store::MAX_SIZE )]
    pub sol_store: Account<'info, Store>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut, has_one = signer)]
    pub sol_store: Account<'info, Store>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolAndCloseAccount<'info> {
    #[account(mut, has_one = signer, close = signer, constraint = sol_store.is_unlocked() @ DiamondHandingError::LockedStore)]
    pub sol_store: Account<'info, Store>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitSplStore<'info> {
    // this is a bit like defining the function types and constraints before implementing it.
    #[account(init, seeds = [b"spl", signer.key().as_ref(), mint.key().as_ref()], bump, payer = signer, space = 8 + Store::MAX_SIZE )]
    pub spl_store: Account<'info, Store>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitAssociatedTokenAccount<'info> {
    // this is a bit like defining the function types and constraints before implementing it.
    #[account(seeds = [b"spl", signer.key().as_ref(), mint.key().as_ref()], bump, has_one = signer, )]
    pub spl_store: Account<'info, Store>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = spl_store,
    )]
    pub token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSplToken<'info> {
    #[account(mut, has_one = signer)]
    pub spl_store: Account<'info, Store>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = spl_store,)]
    pub to_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = signer,)]
    pub from_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawSplAndCloseAccounts<'info> {
    #[account(mut, seeds = [b"spl", signer.key().as_ref(), mint.key().as_ref()], bump, has_one = signer, close = signer, constraint = spl_store.is_unlocked() @ DiamondHandingError::LockedStore)]
    pub spl_store: Account<'info, Store>,
    #[account(mut,
        associated_token::mint = mint,
        associated_token::authority = spl_store)]
    pub store_ata: Account<'info, TokenAccount>,
    #[account(mut, associated_token::mint = mint, associated_token::authority = signer,)]
    pub signer_ata: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
