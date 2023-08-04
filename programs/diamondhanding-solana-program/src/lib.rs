use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("5Zm2UQMSM63NLJGkQYP6xqqGm2EPzYyVNtyPpJnJb5iD");

#[program]
mod hello_world {
    use super::*;
    pub fn say_hello(ctx: Context<SayHello>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;
        msg!("Hello World! - Greeting # {}", counter.count);
        Ok(())
    }
    pub fn initialize_counter(ctx: Context<Initialize>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = 0;
        msg!("Initialized new count. Current value: {}!", counter.count);
        Ok(())
    }
}

#[account]
pub struct Counter {
    // this means that the Account will be known as a Counter and it will store a count data for us.
    // This is a bit like a schema for a MongoDB model. 1 Account = 1 document.
    // Naturally the public key is the document id already.
    count: u64,
}

#[derive(Accounts)]
pub struct SayHello<'info> {
    // this function will take in a input which is the public key of the counter account
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // this is a bit like defining the function types and constraints before implementing it.
    #[account(init, payer = signer, space = 8 + 8)]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
