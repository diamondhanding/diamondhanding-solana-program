use anchor_lang::{prelude::*, solana_program::clock};
use std::convert::TryInto;

#[account]
pub struct Store {
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
