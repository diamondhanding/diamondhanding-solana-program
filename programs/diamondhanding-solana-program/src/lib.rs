use anchor_lang::prelude::*;

declare_id!("5Zm2UQMSM63NLJGkQYP6xqqGm2EPzYyVNtyPpJnJb5iD");

#[program]
pub mod diamondhanding_solana_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
