# GeneLedger

**Verifiable species identification on Solana, then from DNA to protein to 3D structure.**

Built in a 30-minute Solana workshop by a molecular biology student, using Claude Code.

## The idea in plain words

A lab sequences DNA from an unknown bacterium and wants to know what species it is. Today the reference data and the answer live in private files that anyone with access can edit.

GeneLedger puts them on the Solana blockchain instead:

- **Reference sequences** from known species are stored on-chain as a shared database.
- **An unknown read** is compared against them on-chain, and the best match and its score are recorded with the submitter's wallet and a timestamp.
- **The DNA is translated into its protein** on-chain, using the standard genetic code.

The result is a public lab notebook that cannot be quietly changed, and anyone can re-check it.

## Try the demo (1 minute)

1. Download this repository (green **Code** button, then **Download ZIP**) and open `app/index.html` in a browser.
   Or try it without downloading: https://htmlpreview.github.io/?https://github.com/shaanbhasin64-ux/geneledger/blob/main/app/index.html
2. Click **Load example sequence**, then **Run pipeline**.
3. Scroll through the five steps.

The example is real DNA: the first 180 letters of the *rpoB* gene from *E. coli* O157:H7 str. Sakai (NCBI NP_312937.1). You can also paste it yourself:

```
ATGGTTTACTCCTATACCGAGAAAAAACGTATTCGTAAGGATTTTGGTAAACGTCCACAAGTTCTGGATGTACCTTATCTCCTTTCTATCCAGCTTGACTCGTTTCAGAAATTTATCGAGCAAGATCCTGAAGGGCAGTATGGTCTGGAAGCTGCTTTCCGTTCCGTATTCCCGATTCAG
```

## What the five steps show

| Step | What happens | Where it belongs |
|---|---|---|
| 1. Unknown read | You paste a DNA sequence (letters A, C, G, T) | Input |
| 2. Identify species | The read is lined up against known species and matching letters are counted. Highest score wins | Solana program |
| 3. Translate | DNA is read three letters at a time; each triplet codes for one amino acid, and the chain of amino acids is the protein | Solana program |
| 4. Secondary structure | A statistical estimate of which parts of the chain curl into spirals or flatten into strands | Off-chain (web page) |
| 5. 3D structure | The real, lab-solved structure of the protein machine this gene helps build (RNA polymerase, PDB 4YG2) | Off-chain (web page) |

Small, checkable steps go on-chain. Heavy computation stays off-chain.

## How Solana is used

The program is in `programs/gene_ledger/`, written in Rust with the Anchor framework. It has three instructions:

- `add_reference(species, seq)` stores a reference sequence at an address derived from the species name, so a species cannot be registered twice.
- `identify(read)` compares the read with the reference accounts passed in, and stores the best species, the identity score (in basis points) and a timestamp under the submitter's wallet. It only accepts references that belong to this program.
- `translate(seq)` stores the translated protein under the submitter's wallet.

Every instruction requires a wallet signature, rejects anything that is not A/C/G/T or is over 300 letters, and emits an event.

## Honest status

- The Solana program is **written and passed a security lint, but it has not been built, tested or deployed.** The toolchain did not finish installing within the time limit.
- The web page is **not connected to Solana.** It runs the same calculations in the browser to show what the program does.
- Tests are written in `programs/gene_ledger/tests/` but have not been run.
- The identity comparison is a simplified BLAST: no gaps, position by position.
- Steps 4 and 5 are an estimate and a display of a known structure, not protein folding from scratch.

Next step: build, test, deploy to devnet, and have the page read results from the chain.

## Data sources

- Reference sequences: first 180 bases of *rpoB* from NCBI RefSeq. *E. coli* K-12 MG1655 (NP_418414.1), *Salmonella* Typhimurium LT2 (NP_463022.1), *Bacillus subtilis* 168 (NP_387988.2).
- 3D structure: RCSB PDB entry 4YG2, *E. coli* RNA polymerase holoenzyme.

## Repository layout

```
app/index.html                  the visual demo (single file)
programs/gene_ledger/src/       the Solana program
programs/gene_ledger/tests/     its tests
.claude/, CLAUDE.md             the workshop's Claude Code setup
```
