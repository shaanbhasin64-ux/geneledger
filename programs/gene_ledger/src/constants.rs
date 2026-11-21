pub const REF_SEED: &[u8] = b"ref";
pub const ID_SEED: &[u8] = b"id";
pub const PROTEIN_SEED: &[u8] = b"protein";

/// Sequences are ASCII A/C/G/T. Kept short for tx-size and compute limits.
pub const MAX_SEQ_LEN: usize = 300;
pub const MAX_SPECIES_LEN: usize = 32;
pub const MAX_PROTEIN_LEN: usize = MAX_SEQ_LEN / 3;
/// Cap on reference accounts compared in one `identify` call (bounded loop).
pub const MAX_REFERENCES: usize = 8;
pub const BPS_DENOMINATOR: u64 = 10_000;
