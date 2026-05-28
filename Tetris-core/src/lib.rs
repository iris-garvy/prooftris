#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub rows: [u16; 25],
}

impl Board {
    pub const WIDTH: usize = 10;
    pub const HEIGHT: usize = 25;
    pub const INVISIBLE_HEIGHT: usize = 5;
    pub const FULL_ROW: u16 = 0b1111111111;

    pub fn new() -> Self {
        Board { rows: [0; 25] }
    }

    pub fn no_collision(&self, shape: &[u16; 4], row: i32, col: i32) -> bool {
        for (i, &piece_row) in shape.iter().enumerate() {
            if piece_row == 0 {
                continue;
            }
            let board_row = row + i as i32;
            if board_row < 0 || board_row >= Self::HEIGHT as i32 {
                return false;
            }

            let shifted = if col >= 0 {
                piece_row << col
            } else {
                let shift = (-col) as u32;
                let mask = (1u16 << shift) - 1;
                if (piece_row & mask) != 0 {
                    return false;
                }
                piece_row >> shift
            };

            if shifted >= (1 << Self::WIDTH) {
                return false;
            }
            if (shifted & self.rows[board_row as usize]) != 0 {
                return false;
            }
        }
        true
    }

    pub fn place(&mut self, state: &PieceState) {
        let shape = state.shape();
        for (i, &piece_row) in shape.iter().enumerate() {
            if piece_row == 0 {
                continue;
            }
            let board_row = state.row + i as i32;
            if board_row < 0 || board_row >= Self::HEIGHT as i32 {
                continue;
            }
            let shifted = if state.col >= 0 {
                piece_row << state.col
            } else {
                piece_row >> (-state.col)
            };
            self.rows[board_row as usize] |= shifted;
        }
    }

    pub fn clear_lines(&mut self) -> usize {
        let mut cleared = 0;
        let mut write_row = Self::HEIGHT - 1;

        for read_row in (0..Self::HEIGHT).rev() {
            if self.rows[read_row] == Self::FULL_ROW {
                cleared += 1;
            } else {
                if write_row != read_row {
                    self.rows[write_row] = self.rows[read_row];
                }
                if write_row > 0 {
                    write_row -= 1;
                }
            }
        }
        for r in 0..=write_row {
            self.rows[r] = 0;
        }
        cleared
    }

    pub fn check_pc(&self) -> bool {
        self.rows == [0; 25]
    }
}

pub const KICK_JLSZT: [[(i32, i32); 5]; 8] = [
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
];

pub const KICK_I: [[(i32, i32); 5]; 8] = [
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
];

pub const KICK_O: [[(i32, i32); 5]; 8] = [[(0, 0); 5]; 8];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    I,
    O,
    T,
    S,
    Z,
    L,
    J,
}

impl Piece {
    /// All shapes are **left‑aligned**: bit 0 = leftmost column of the piece.
    pub fn shapes(&self) -> [[u16; 4]; 4] {
        match self {
            Piece::I => [
                [0b0000, 0b1111, 0b0000, 0b0000], // 0
                [0b100, 0b100, 0b100, 0b100],     // 1
                [0b0000, 0b0000, 0b1111, 0b0000], // 2
                [0b10, 0b10, 0b10, 0b10],         // 3
            ],
            Piece::O => [[0b11, 0b11, 0b0000, 0b0000]; 4],
            Piece::T => [
                [0b010, 0b111, 0b000, 0b000], // 0
                [0b010, 0b110, 0b010, 0b000], // 1
                [0b000, 0b111, 0b010, 0b000], // 2
                [0b010, 0b011, 0b010, 0b000], // 3
            ],
            Piece::S => [
                [0b110, 0b011, 0b000, 0b000], // 0
                [0b010, 0b110, 0b100, 0b000], // 1
                [0b000, 0b110, 0b011, 0b000], // 2
                [0b001, 0b011, 0b010, 0b000], // 3
            ],
            Piece::Z => [
                [0b011, 0b110, 0b000, 0b000], // 0
                [0b100, 0b110, 0b010, 0b000], // 1
                [0b000, 0b011, 0b110, 0b000], // 2
                [0b010, 0b011, 0b001, 0b000], // 3
            ],
            Piece::J => [
                [0b001, 0b111, 0b000, 0b000], // 0
                [0b110, 0b010, 0b010, 0b000], // 1
                [0b000, 0b111, 0b100, 0b000], // 2
                [0b010, 0b010, 0b011, 0b000], // 3
            ],
            Piece::L => [
                [0b100, 0b111, 0b000, 0b000], // 0
                [0b010, 0b010, 0b110, 0b000], // 1
                [0b000, 0b111, 0b001, 0b000], // 2
                [0b011, 0b010, 0b010, 0b000], // 3
            ],
        }
    }

    pub fn shape(&self, rotation: u8) -> [u16; 4] {
        self.shapes()[(rotation % 4) as usize]
    }

    pub fn kick_table(&self) -> &'static [[(i32, i32); 5]; 8] {
        match self {
            Piece::I => &KICK_I,
            Piece::O => &KICK_O,
            _ => &KICK_JLSZT,
        }
    }

    pub fn kicks(&self, from: u8, dir: i32) -> &'static [(i32, i32); 5] {
        let base = (from % 4) as usize * 2;
        let idx = if dir == 1 { base + 1 } else { base };
        &self.kick_table()[idx]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PieceState {
    pub piece: Piece,
    pub rotation: u8,
    pub row: i32,
    pub col: i32,
}

impl PieceState {
    pub fn spawn(piece: Piece) -> Self {
        let col = match piece {
            Piece::O => 4,
            _ => 3,
        };
        PieceState {
            piece,
            rotation: 0,
            row: 5,
            col,
        }
    }

    pub fn shape(&self) -> [u16; 4] {
        self.piece.shape(self.rotation)
    }

    pub fn move_horizontal(&mut self, board: &Board, dx: i32) -> bool {
        let new_col = self.col + dx;
        if board.no_collision(&self.shape(), self.row, new_col) {
            self.col = new_col;
            true
        } else {
            false
        }
    }

    pub fn soft_drop(&mut self, board: &Board) -> bool {
        let new_row = self.row + 1;
        if board.no_collision(&self.shape(), new_row, self.col) {
            self.row = new_row;
            true
        } else {
            false
        }
    }

    pub fn hard_drop(&mut self, board: &Board) -> bool {
        let initial_row = self.row;
        let mut final_row = self.row;
        while board.no_collision(&self.shape(), final_row + 1, self.col) {
            final_row += 1;
        }
        self.row = final_row;
        initial_row != final_row
    }

    pub fn rotate(&mut self, board: &Board, dir: i32) -> bool {
        let from = self.rotation;
        let to = ((from as i32 + dir).rem_euclid(4)) as u8;
        let new_shape = self.piece.shape(to);
        let kicks = self.piece.kicks(from, dir);

        for &(dx, dy) in kicks.iter() {
            let new_col = self.col + dx;
            let new_row = self.row - dy;
            if board.no_collision(&new_shape, new_row, new_col) {
                self.rotation = to;
                self.row = new_row;
                self.col = new_col;
                return true;
            }
        }
        false
    }

    pub fn check_immobile(&self, board: &Board) -> bool {
        let shape = self.shape();
        let col = self.col;
        let row = self.row;
        if board.no_collision(&shape, row + 1, col) {
            return false;
        } else if board.no_collision(&shape, row - 1, col) {
            return false;
        } else if board.no_collision(&shape, row, col + 1) {
            return false;
        } else if board.no_collision(&shape, row, col - 1) {
            return false;
        }
        true
    }

    pub fn three_corners(&self, board: &Board) -> bool {
        if self.piece != Piece::T {
            return false;
        }
        let corners = [
            (self.row, self.col),
            (self.row + 2, self.col),
            (self.row, self.col + 2),
            (self.row + 2, self.col + 2),
        ];
        let mut filled = 0;
        for &(row, col) in &corners {
            if row < 0 || row >= Board::HEIGHT as i32 || col < 0 || col >= Board::WIDTH as i32 {
                filled += 1; // wall = filled
            } else if (board.rows[row as usize] >> col) & 1 == 1 {
                filled += 1;
            }
        }
        filled >= 3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub tsd: u32,
    pub tst: u32,
    pub tss: u32,
    pub tetris: u32,
    pub pc: u32,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            tsd: 0,
            tst: 0,
            tss: 0,
            tetris: 0,
            pc: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    ShiftLeft,
    ShiftRight,
    RotateCW,
    RotateCCW,
    SoftDrop,
    HardDrop,
    Place,
}

pub fn simulate(
    ledger: &mut Ledger,
    board: &mut Board,
    pieces: &[Piece],
    all_actions: &[Vec<Action>],
) -> Result<(), &'static str> {
    for (piece, actions) in pieces.iter().zip(all_actions.iter()) {
        let mut state = PieceState::spawn(*piece);
        let mut previous_action = Action::Place;
        let mut tspin = false;
        for &action in actions {
            match action {
                Action::ShiftLeft => {
                    if state.move_horizontal(board, -1) {
                        previous_action = Action::ShiftLeft;
                    }
                }
                Action::ShiftRight => {
                    if state.move_horizontal(board, 1) {
                        previous_action = Action::ShiftRight;
                    }
                }
                Action::RotateCW => {
                    if state.rotate(board, 1) {
                        previous_action = Action::RotateCW;
                    }
                }
                Action::RotateCCW => {
                    if state.rotate(board, -1) {
                        previous_action = Action::RotateCCW;
                    }
                }
                Action::SoftDrop => {
                    if state.soft_drop(board) {
                        previous_action = Action::SoftDrop;
                    }
                }
                Action::HardDrop => {
                    if state.hard_drop(board) {
                        previous_action = Action::HardDrop;
                    }
                }
                Action::Place => {
                    if state.hard_drop(board) {
                        previous_action = Action::HardDrop;
                    }
                    if (previous_action == Action::RotateCCW || previous_action == Action::RotateCW)
                        && state.three_corners(&board)
                    {
                        tspin = true;
                    }
                    board.place(&state);
                    let cleared_lines = board.clear_lines();
                    if cleared_lines > 0 {
                        if tspin {
                            match cleared_lines {
                                1 => {
                                    ledger.tss += 1;
                                }
                                2 => {
                                    ledger.tsd += 1;
                                }
                                3 => {
                                    ledger.tst += 1;
                                }
                                _ => {}
                            }
                        }
                        if cleared_lines >= 4 {
                            ledger.tetris += 1;
                        }
                        if board.check_pc() {
                            ledger.pc += 1;
                        }
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use alloc::vec;

//     #[test]
//     fn test_visual_output() {
//         let mut board = Board::new();
//         let mut ledger = Ledger::new();
//         let pieces = vec![Piece::O, Piece::I, Piece::I, Piece::O, Piece::O];
//         let actions = vec![
//             vec![Action::ShiftLeft, Action::ShiftLeft, Action::ShiftLeft, Action::ShiftLeft, Action::Place],
//             vec![Action::ShiftLeft, Action::Place],
//             vec![Action::ShiftLeft, Action::Place],
//             vec![Action::ShiftRight, Action::ShiftRight,
//             Action::ShiftRight, Action::ShiftRight, Action::RotateCCW,
//             Action::HardDrop, Action::RotateCCW, Action::Place],
//             vec![Action::ShiftRight, Action::ShiftRight, Action::Place],
//         ];
//         simulate(&mut ledger, &mut board, &pieces, &actions).unwrap();

//         println!("\nBoard state (visible rows 5-24):");
//         for (i, row) in board.rows.iter().enumerate().skip(Board::INVISIBLE_HEIGHT) {
//             print!("{:2}: ", i);
//             for col in 0..10 {
//                 print!("{}", if (row >> col) & 1 == 1 { '#' } else { '.' });
//             }
//             println!();
//         }
//         println!("{:?}",ledger);
//     }
// }
