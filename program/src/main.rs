#![no_main]
sp1_zkvm::entrypoint!(main);

use tetris_core::{simulate, Action, Board, Ledger, Piece};

pub fn main() {
    let initial_board = sp1_zkvm::io::read::<Board>();
    let pieces = sp1_zkvm::io::read::<Vec<Piece>>();

    let actions = sp1_zkvm::io::read::<Vec<Vec<Action>>>();

    let mut board = initial_board;
    let mut ledger = Ledger::new();
    simulate(&mut ledger, &mut board, &pieces, &actions).expect("simulation failed");

    sp1_zkvm::io::commit(&ledger);
}
