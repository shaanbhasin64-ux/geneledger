//! GeneLedger: verifiable species identification + DNA -> protein translation.
//! Workshop demo. Sequences are short ASCII A/C/G/T fragments.
use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod gene_ledger {
    use super::*;

    /// Store a known species' reference sequence in the shared ledger.
    pub fn add_reference(ctx: Context<AddReference>, species: String, seq: Vec<u8>) -> Result<()> {
        instructions::add_reference::handle_add_reference(ctx, species, seq)
    }

    /// Compare an unknown read against the passed reference accounts and
    /// record the best match and its % identity (basis points).
    pub fn identify(ctx: Context<Identify>, read: Vec<u8>) -> Result<()> {
        instructions::identify::handle_identify(ctx, read)
    }

    /// Translate DNA into its amino-acid chain (primary structure).
    pub fn translate(ctx: Context<Translate>, seq: Vec<u8>) -> Result<()> {
        instructions::translate::handle_translate(ctx, seq)
    }
}
