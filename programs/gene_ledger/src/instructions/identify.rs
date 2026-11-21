use anchor_lang::prelude::*;

use crate::{constants::*, errors::GeneLedgerError, events::Identified, state::*};

#[derive(Accounts)]
pub struct Identify<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,
    #[account(
        init,
        payer = submitter,
        space = 8 + Identification::INIT_SPACE,
        seeds = [ID_SEED, submitter.key().as_ref()],
        bump
    )]
    pub identification: Account<'info, Identification>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: 1..=MAX_REFERENCES read-only `Reference` accounts.
}

pub fn handle_identify(ctx: Context<Identify>, read: Vec<u8>) -> Result<()> {
    validate_seq(&read)?;
    let refs = ctx.remaining_accounts;
    require!(
        !refs.is_empty() && refs.len() <= MAX_REFERENCES,
        GeneLedgerError::InvalidReferenceCount
    );

    let mut best_species = String::new();
    let mut best_bps: u16 = 0;
    for info in refs.iter() {
        // `Account::try_from` checks program ownership + the Reference discriminator,
        // so only genuine ledger references can be compared against.
        let reference = Account::<Reference>::try_from(info)?;
        let bps = identity_bps(&read, &reference.seq)?;
        if best_species.is_empty() || bps > best_bps {
            best_bps = bps;
            best_species = reference.species.clone();
        }
    }

    let timestamp = Clock::get()?.unix_timestamp;
    let id = &mut ctx.accounts.identification;
    id.submitter = ctx.accounts.submitter.key();
    id.best_species = best_species.clone();
    id.identity_bps = best_bps;
    id.timestamp = timestamp;
    id.bump = ctx.bumps.identification;

    emit!(Identified {
        submitter: id.submitter,
        best_species,
        identity_bps: best_bps,
        timestamp,
    });
    Ok(())
}
