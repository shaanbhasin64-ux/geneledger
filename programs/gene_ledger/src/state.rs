use anchor_lang::prelude::*;

use crate::{constants::*, errors::GeneLedgerError};

/// A known species' reference sequence. PDA: ["ref", species].
#[account]
#[derive(InitSpace)]
pub struct Reference {
    pub authority: Pubkey,
    #[max_len(MAX_SPECIES_LEN)]
    pub species: String,
    #[max_len(MAX_SEQ_LEN)]
    pub seq: Vec<u8>,
    pub bump: u8,
}

/// Result of a species identification. PDA: ["id", submitter].
#[account]
#[derive(InitSpace)]
pub struct Identification {
    pub submitter: Pubkey,
    #[max_len(MAX_SPECIES_LEN)]
    pub best_species: String,
    /// Percent identity in basis points (9750 = 97.50%).
    pub identity_bps: u16,
    pub timestamp: i64,
    pub bump: u8,
}

/// Translated primary structure. PDA: ["protein", submitter].
#[account]
#[derive(InitSpace)]
pub struct Protein {
    pub submitter: Pubkey,
    #[max_len(MAX_PROTEIN_LEN)]
    pub protein: String,
    pub bump: u8,
}

/// Only A/C/G/T, non-empty, within the length cap.
pub fn validate_seq(seq: &[u8]) -> Result<()> {
    require!(!seq.is_empty(), GeneLedgerError::EmptySequence);
    require!(seq.len() <= MAX_SEQ_LEN, GeneLedgerError::SequenceTooLong);
    require!(
        seq.iter().all(|b| matches!(b, b'A' | b'C' | b'G' | b'T')),
        GeneLedgerError::InvalidBase
    );
    Ok(())
}

/// Simplified BLAST: ungapped, position-by-position identity over the
/// shorter of the two sequences, returned in basis points.
pub fn identity_bps(a: &[u8], b: &[u8]) -> Result<u16> {
    let n = a.len().min(b.len());
    if n == 0 {
        return Ok(0);
    }
    let matches = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    let bps = (matches as u64)
        .checked_mul(BPS_DENOMINATOR)
        .and_then(|v| v.checked_div(n as u64))
        .ok_or(GeneLedgerError::MathOverflow)?;
    u16::try_from(bps).map_err(|_| GeneLedgerError::MathOverflow.into())
}

/// Standard genetic code, indexed by base order T, C, A, G.
const CODON_TABLE: &[u8; 64] = b"FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG";

fn base_index(b: u8) -> Result<usize> {
    match b {
        b'T' => Ok(0),
        b'C' => Ok(1),
        b'A' => Ok(2),
        b'G' => Ok(3),
        _ => err!(GeneLedgerError::InvalidBase),
    }
}

/// Translate DNA to one-letter amino acids, stopping at the first stop codon.
pub fn translate_dna(seq: &[u8]) -> Result<String> {
    let mut protein = String::with_capacity(seq.len() / 3);
    for codon in seq.chunks_exact(3) {
        let idx = base_index(codon[0])? * 16 + base_index(codon[1])? * 4 + base_index(codon[2])?;
        let aa = CODON_TABLE[idx];
        if aa == b'*' {
            break;
        }
        protein.push(aa as char);
    }
    Ok(protein)
}
