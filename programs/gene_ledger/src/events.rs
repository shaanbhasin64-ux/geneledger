use anchor_lang::prelude::*;

#[event]
pub struct ReferenceAdded {
    pub authority: Pubkey,
    pub species: String,
    pub length: u16,
}

#[event]
pub struct Identified {
    pub submitter: Pubkey,
    pub best_species: String,
    pub identity_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct Translated {
    pub submitter: Pubkey,
    pub protein: String,
}
