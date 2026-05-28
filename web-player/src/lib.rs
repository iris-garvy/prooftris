use tetris_core::{Board, Ledger, Piece, PieceState};
use wasm_bindgen::prelude::*;

const PIECES: [Piece; 7] = [
    Piece::I,
    Piece::O,
    Piece::T,
    Piece::S,
    Piece::Z,
    Piece::J,
    Piece::L,
];

#[wasm_bindgen]
pub struct Game {
    board: Board,
    current: Option<PieceState>,
    next: Vec<Piece>,
    ledger: Ledger,
    game_over: bool,
    color: [u8; 250],
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Game {
        console_error_panic_hook::set_once();
        let mut game = Game {
            board: Board::new(),
            current: None,
            next: Vec::new(),
            ledger: Ledger::new(),
            game_over: false,
            color: [0u8; 250],
        };
        game.fill_bag();
        game.spawn_next();
        game
    }

    pub fn move_left(&mut self) {
        self.with_current(|state, board| {
            state.move_horizontal(board, -1);
        });
    }

    pub fn move_right(&mut self) {
        self.with_current(|state, board| {
            state.move_horizontal(board, 1);
        });
    }

    pub fn rotate_cw(&mut self) {
        self.with_current(|state, board| {
            state.rotate(board, 1);
        });
    }

    pub fn rotate_ccw(&mut self) {
        self.with_current(|state, board| {
            state.rotate(board, -1);
        });
    }

    pub fn soft_drop(&mut self) -> bool {
        self.with_current(|state, board| state.soft_drop(board))
    }

    pub fn hard_drop(&mut self) {
        if let Some(mut state) = self.current.take() {
            state.hard_drop(&mut self.board);
            self.lock_and_spawn(&state);
        }
    }

    pub fn board_colors(&self) -> Vec<u8> {
        let mut colors = self.color.clone();

        if let Some(ref state) = self.current {
            let shape = state.shape();
            for (i, &piece_row) in shape.iter().enumerate() {
                if piece_row == 0 {
                    continue;
                }
                let board_row = state.row + i as i32;
                if board_row < 0 || board_row >= 25 {
                    continue;
                }
                let mut shifted = piece_row;
                if state.col >= 0 {
                    shifted <<= state.col;
                } else {
                    shifted >>= -state.col;
                }
                for col in 0..10 {
                    if (shifted >> col) & 1 == 1 {
                        let idx = board_row as usize * 10 + col as usize;
                        colors[idx] = state.piece as u8 + 1;
                    }
                }
            }
        }
        colors[50..250].to_vec()
    }

    pub fn score(&self) -> u32 {
        self.ledger.tetris * 4 + self.ledger.tss + self.ledger.tsd * 2 + self.ledger.tst * 3
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn next_piece(&self) -> u8 {
        self.next.last().map(|p| *p as u8).unwrap_or(0)
    }
}

impl Game {
    /// Runs a closure on the current piece and the board.
    /// Returns `R::default()` if there is no current piece.
    fn with_current<F, R: Default>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut PieceState, &mut Board) -> R,
    {
        if let Some(ref mut state) = self.current {
            f(state, &mut self.board)
        } else {
            R::default()
        }
    }

    fn fill_bag(&mut self) {
        let mut bag: Vec<Piece> = PIECES.to_vec();
        for i in (1..bag.len()).rev() {
            let j = fastrand::usize(..=i);
            bag.swap(i, j);
        }
        // prepend new bag to existing `next` list
        let mut new_bag = bag;
        new_bag.extend(self.next.drain(..));
        self.next = new_bag;
    }

    fn spawn_next(&mut self) {
        if self.next.is_empty() {
            self.fill_bag();
        }
        let piece = self.next.pop().unwrap();
        let state = PieceState::spawn(piece);
        if self
            .board
            .no_collision(&state.shape(), state.row, state.col)
        {
            self.current = Some(state);
        } else {
            self.game_over = true;
            self.current = None;
        }
    }

    fn lock_and_spawn(&mut self, state: &PieceState) {
        // Write piece type into colour grid
        let shape = state.shape();
        let piece_type = state.piece as u8 + 1;
        for (i, &piece_row) in shape.iter().enumerate() {
            if piece_row == 0 {
                continue;
            }
            let board_row = state.row + i as i32;
            if board_row < 0 || board_row >= 25 {
                continue;
            }
            let shifted = if state.col >= 0 {
                piece_row << state.col
            } else {
                piece_row >> (-state.col)
            };
            for col in 0..10 {
                if (shifted >> col) & 1 == 1 {
                    let idx = board_row as usize * 10 + col as usize;
                    self.color[idx] = piece_type;
                }
            }
        }

        self.board.place(state);
        self.shift_colors_down_after_clear();
        self.spawn_next();
    }

    fn shift_colors_down_after_clear(&mut self) {
        let mut new_color = [0u8; 250];
        let mut write_row = 24;
        for read_row in (0..25).rev() {
            if self.board.rows[read_row] == Board::FULL_ROW {
                // cleared row → skip
                continue;
            } else {
                let src = read_row * 10;
                let dst = write_row * 10;
                new_color[dst..dst + 10].copy_from_slice(&self.color[src..src + 10]);
                if write_row > 0 {
                    write_row -= 1;
                }
            }
        }
        self.color = new_color;
    }
}
