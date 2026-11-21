use anchor_lang::prelude::*;

#[error_code]
pub enum GeneLedgerError {
    #[msg("Sequence is empty")]
    EmptySequence,
    #[msg("Sequence exceeds the maximum length")]
    SequenceTooLong,
    #[msg("Sequence may only contain the bases A, C, G, T")]
    InvalidBase,
    #[msg("Species name is empty or too long")]
    InvalidSpeciesName,
    #[msg("Pass between 1 and 8 reference accounts")]
    InvalidReferenceCount,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}
