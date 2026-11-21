use anchor_lang::prelude::*;

use crate::{constants::*, errors::GeneLedgerError, events::ReferenceAdded, state::*};

#[derive(Accounts)]
#[instruction(species: String)]
pub struct AddReference<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Reference::INIT_SPACE,
        seeds = [REF_SEED, species.as_bytes()],
        bump
    )]
    pub reference: Account<'info, Reference>,
    pub system_program: Program<'info, System>,
}

pub fn handle_add_reference(ctx: Context<AddReference>, species: String, seq: Vec<u8>) -> Result<()> {
    require!(
        !species.is_empty() && species.len() <= MAX_SPECIES_LEN,
        GeneLedgerError::InvalidSpeciesName
    );
    validate_seq(&seq)?;

    let length = u16::try_from(seq.len()).map_err(|_| GeneLedgerError::MathOverflow)?;
    let reference = &mut ctx.accounts.reference;
    reference.authority = ctx.accounts.authority.key();
    reference.species = species.clone();
    reference.seq = seq;
    reference.bump = ctx.bumps.reference;

    emit!(ReferenceAdded {
        authority: reference.authority,
        species,
        length,
    });
    Ok(())
}
