use anchor_lang::prelude::*;

#[error_code]
pub enum DiamondHandingError {
    #[msg("Store is LOCKED")]
    LockedStore,
}
