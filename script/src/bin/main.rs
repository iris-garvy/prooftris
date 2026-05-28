//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};
use tetris_core::{Action, Board, Ledger, Piece};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
const PROOFTRIS_ELF: Elf = include_elf!("prooftris-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    let initial_board = Board::new();
    let pieces = vec![Piece::O, Piece::I, Piece::I, Piece::O, Piece::O];
    let requirements = Ledger::new(&[0,0,0,0,1]);

    let actions = vec![
        vec![
            Action::ShiftLeft,
            Action::ShiftLeft,
            Action::ShiftLeft,
            Action::ShiftLeft,
            Action::Place,
        ],
        vec![Action::ShiftLeft, Action::Place],
        vec![Action::ShiftLeft, Action::Place],
        vec![
            Action::ShiftRight,
            Action::ShiftRight,
            Action::ShiftRight,
            Action::ShiftRight,
            Action::RotateCCW,
            Action::HardDrop,
            Action::RotateCCW,
            Action::Place,
        ],
        vec![Action::ShiftRight, Action::ShiftRight, Action::Place],
    ];

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_board);
    stdin.write(&pieces);
    stdin.write(&requirements);
    stdin.write(&actions);

    if args.execute {
        // Execute the program
        let (mut output, report) = client.execute(PROOFTRIS_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let validity: bool = output.read();
        println!("Passes requirements: {:?}", validity);
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let pk = client.setup(PROOFTRIS_ELF).expect("failed to setup elf");

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        client
            .verify(&proof, pk.verifying_key(), None)
            .expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
