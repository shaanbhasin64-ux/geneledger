use anchor_lang::prelude::*;

use crate::{constants::*, events::Translated, state::*};

#[derive(Accounts)]
pub struct Translate<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    #[account(
        init,
        payer = submitter,
        space = 8 + Protein::INIT_SPACE,
        seeds = [PROTEIN_SEED, submitter.key().as_ref()],
        bump
    )]
    pub protein: Account<'info, Protein>,
    pub system_program: Program<'info, System>,
}

pub fn handle_translate(ctx: Context<Translate>, seq: Vec<u8>) -> Result<()> {
    validate_seq(&seq)?;
    let protein_str = translate_dna(&seq)?;

    let protein = &mut ctx.accounts.protein;
    protein.submitter = ctx.accounts.submitter.key();
    protein.protein = protein_str.clone();
    protein.bump = ctx.bumps.protein;

    emit!(Translated {
        submitter: protein.submitter,
        protein: protein_str,
    });
    Ok(())
}
