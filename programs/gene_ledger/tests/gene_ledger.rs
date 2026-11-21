//! LiteSVM tests for gene_ledger. All sequences are SYNTHETIC demo data.
//! Build the program first: `cargo build-sbf` (or `anchor build`).
use anchor_lang::{
    solana_program::instruction::Instruction, system_program, AccountDeserialize, InstructionData,
    ToAccountMetas,
};
use gene_ledger::state::{Identification, Protein};
use litesvm::LiteSVM;
use solana_instruction::AccountMeta;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

const PROGRAM_SO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/gene_ledger.so");

/// Deterministic synthetic sequence (tiny LCG), not real organism data.
fn synth(seed: u64, len: usize) -> Vec<u8> {
    let mut s = seed;
    (0..len)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            b"ACGT"[(s >> 33) as usize % 4]
        })
        .collect()
}

/// Introduce `n` point mutations at evenly spaced positions.
fn mutate(seq: &[u8], n: usize) -> Vec<u8> {
    let mut out = seq.to_vec();
    for k in 0..n {
        let i = (k * seq.len()) / n + 3;
        out[i] = match out[i] {
            b'A' => b'C',
            b'C' => b'G',
            b'G' => b'T',
            _ => b'A',
        };
    }
    out
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(gene_ledger::ID, PROGRAM_SO)
        .expect("build the program first (cargo build-sbf)");
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn send(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction) -> Result<u64, String> {
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
        .map(|m| m.compute_units_consumed)
        .map_err(|e| format!("{:?} {}", e.err, e.meta.logs.join("\n")))
}

fn ref_pda(species: &str) -> Pubkey {
    Pubkey::find_program_address(&[b"ref", species.as_bytes()], &gene_ledger::ID).0
}

fn add_reference_ix(payer: &Pubkey, species: &str, seq: Vec<u8>) -> Instruction {
    Instruction {
        program_id: gene_ledger::ID,
        accounts: gene_ledger::accounts::AddReference {
            authority: *payer,
            reference: ref_pda(species),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: gene_ledger::instruction::AddReference { species: species.to_string(), seq }.data(),
    }
}

fn identify_ix(payer: &Pubkey, read: Vec<u8>, refs: &[Pubkey]) -> Instruction {
    let mut accounts = gene_ledger::accounts::Identify {
        submitter: *payer,
        identification: Pubkey::find_program_address(&[b"id", payer.as_ref()], &gene_ledger::ID).0,
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    accounts.extend(refs.iter().map(|r| AccountMeta::new_readonly(*r, false)));
    Instruction {
        program_id: gene_ledger::ID,
        accounts,
        data: gene_ledger::instruction::Identify { read }.data(),
    }
}

fn translate_ix(payer: &Pubkey, seq: Vec<u8>) -> Instruction {
    Instruction {
        program_id: gene_ledger::ID,
        accounts: gene_ledger::accounts::Translate {
            submitter: *payer,
            protein: Pubkey::find_program_address(&[b"protein", payer.as_ref()], &gene_ledger::ID).0,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: gene_ledger::instruction::Translate { seq }.data(),
    }
}

const SPECIES: [&str; 3] = ["Reference A", "Reference B", "Reference C"];

fn add_three_refs(svm: &mut LiteSVM, payer: &Keypair) -> Vec<Vec<u8>> {
    let seqs: Vec<Vec<u8>> = (0..3).map(|i| synth(100 + i, 180)).collect();
    for (name, seq) in SPECIES.iter().zip(&seqs) {
        let cu = send(svm, payer, add_reference_ix(&payer.pubkey(), name, seq.clone())).unwrap();
        println!("add_reference CU: {cu}");
    }
    seqs
}

#[test]
fn identifies_mutated_read_as_reference_b() {
    let (mut svm, payer) = setup();
    let seqs = add_three_refs(&mut svm, &payer);

    // Unknown read = Reference B with 5 point mutations -> 175/180 = 97.22%.
    let read = mutate(&seqs[1], 5);
    let refs: Vec<Pubkey> = SPECIES.iter().map(|s| ref_pda(s)).collect();
    let cu = send(&mut svm, &payer, identify_ix(&payer.pubkey(), read, &refs)).unwrap();
    println!("identify CU (3 refs x 180 bp): {cu}");
    assert!(cu < 100_000, "identify CU budget exceeded: {cu}");

    let pda = Pubkey::find_program_address(&[b"id", payer.pubkey().as_ref()], &gene_ledger::ID).0;
    let acc = svm.get_account(&pda).unwrap();
    let id = Identification::try_deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(id.best_species, "Reference B");
    assert_eq!(id.identity_bps, 9722);
    assert_eq!(id.submitter, payer.pubkey());
}

#[test]
fn translates_known_orf() {
    let (mut svm, payer) = setup();
    // ATG GCC GAA GAA CTG TAA GGG -> M A E E L, stops at TAA.
    let cu = send(&mut svm, &payer, translate_ix(&payer.pubkey(), b"ATGGCCGAAGAACTGTAAGGG".to_vec())).unwrap();
    println!("translate CU: {cu}");
    assert!(cu < 50_000, "translate CU budget exceeded: {cu}");

    let pda = Pubkey::find_program_address(&[b"protein", payer.pubkey().as_ref()], &gene_ledger::ID).0;
    let acc = svm.get_account(&pda).unwrap();
    let protein = Protein::try_deserialize(&mut &acc.data[..]).unwrap();
    assert_eq!(protein.protein, "MAEEL");
}

#[test]
fn rejects_invalid_base() {
    let (mut svm, payer) = setup();
    let err = send(&mut svm, &payer, add_reference_ix(&payer.pubkey(), "Bad", b"ACGTNNACGT".to_vec())).unwrap_err();
    assert!(err.contains("InvalidBase"), "{err}");
    let err = send(&mut svm, &payer, translate_ix(&payer.pubkey(), b"ATGXYZ".to_vec())).unwrap_err();
    assert!(err.contains("InvalidBase"), "{err}");
}

#[test]
fn rejects_empty_and_too_long_sequences() {
    let (mut svm, payer) = setup();
    let err = send(&mut svm, &payer, add_reference_ix(&payer.pubkey(), "Empty", vec![])).unwrap_err();
    assert!(err.contains("EmptySequence"), "{err}");
    let err = send(&mut svm, &payer, add_reference_ix(&payer.pubkey(), "Long", synth(1, 301))).unwrap_err();
    assert!(err.contains("SequenceTooLong"), "{err}");
}

#[test]
fn rejects_duplicate_species() {
    let (mut svm, payer) = setup();
    send(&mut svm, &payer, add_reference_ix(&payer.pubkey(), "Dup", synth(1, 60))).unwrap();
    svm.expire_blockhash();
    assert!(send(&mut svm, &payer, add_reference_ix(&payer.pubkey(), "Dup", synth(2, 60))).is_err());
}

#[test]
fn identify_rejects_missing_or_fake_references() {
    let (mut svm, payer) = setup();
    let err = send(&mut svm, &payer, identify_ix(&payer.pubkey(), synth(5, 60), &[])).unwrap_err();
    assert!(err.contains("InvalidReferenceCount"), "{err}");

    // A system-owned account is not a ledger Reference -> owner check fails.
    let fake = Keypair::new();
    svm.airdrop(&fake.pubkey(), 1_000_000_000).unwrap();
    let err = send(&mut svm, &payer, identify_ix(&payer.pubkey(), synth(5, 60), &[fake.pubkey()])).unwrap_err();
    assert!(err.contains("AccountOwnedByWrongProgram"), "{err}");
}
