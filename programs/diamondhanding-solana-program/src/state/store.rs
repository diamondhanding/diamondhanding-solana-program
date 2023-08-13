use anchor_lang::{prelude::*, solana_program::clock};
use std::convert::TryInto;

#[account]
pub struct Store {
    // this means that the Account will be known as a Counter and it will store a count data for us.
    // This is a bit like a schema for a MongoDB model. 1 Account = 1 document.
    // Naturally the public key is the document id already.
    pub unlock_date: i64,
    pub can_manually_unlock: bool,
    pub signer: Pubkey,
}

impl Store {
    pub fn is_unlocked(&self) -> bool {
        fn now_ts() -> Result<i64> {
            Ok(clock::Clock::get()?.unix_timestamp.try_into().unwrap())
        }
        self.unlock_date <= now_ts().unwrap() || self.can_manually_unlock
    }

    pub const MAX_SIZE: usize = 8 + (1) + (32);
}
