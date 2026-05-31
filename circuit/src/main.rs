use plonky2::field::types::{Field, Field64};
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::field::goldilocks_field::GoldilocksField;

const MAX_PIECES: usize = 30;
const MAX_ACTIONS: usize = 90;

fn simulate(
    builder: &mut CircuitBuilder<GoldilocksField, 2>, 
    board: BoardTargets, 
    queue: Vec<Target>, 
    actions: Vec<Vec<ActionTargets>>
) -> LedgerTargets {
    let mut queue_index = 0;
    let mut board = board;
    let mut ledger = LedgerTargets::empty(builder);
    for piece_actions in &actions {
        let (piece, _) = PieceStateTargets::spawn(queue[queue_index], builder, board);
        let mut game_state = GameState::new(builder, board, piece, ledger);
        for action in piece_actions {
            game_state = game_state.apply_movement(builder, *action);
        }
        game_state = game_state.lock_piece(builder);
        board = game_state.board;
        ledger = game_state.ledger;
        queue_index += 1;
    }
    ledger
}

fn in_range(builder: &mut CircuitBuilder<GoldilocksField, 2>, target: Target, min: usize, max: usize) -> BoolTarget {
    let mut checker = builder._false();
    for idx in min..max {
        let idx_t = builder.constant(GoldilocksField::from_canonical_usize(idx));
        let is_idx = builder.is_equal(idx_t, target);
        checker = builder.or(is_idx, checker);
    }
    checker
}

fn select_piece_state(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    cond: BoolTarget,
    a: PieceStateTargets,
    b: PieceStateTargets,
) -> PieceStateTargets {
    PieceStateTargets {
        piece: builder.select(cond, a.piece, b.piece),
        rotation: builder.select(cond, a.rotation, b.rotation),
        row: builder.select(cond, a.row, b.row),
        col: builder.select(cond, a.col, b.col),
    }
}


fn select_board(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    cond: BoolTarget,
    a: BoardTargets,
    b: BoardTargets,
) -> BoardTargets {
    let mut out = [[builder._false(); 10]; 25];

    for y in 0..25 {
        for x in 0..10 {
            let selected = builder.select(
                cond,
                a.cells[y][x].target,
                b.cells[y][x].target,
            );

            out[y][x] = BoolTarget::new_unsafe(selected);
        }
    }

    BoardTargets { cells: out }
}

#[derive(Debug, Clone, Copy)]
struct GameState {
    board: BoardTargets,
    current_piece: PieceStateTargets,
    last_action_was_rotation: BoolTarget,
    ledger: LedgerTargets
}

impl GameState {
    fn new(
        builder: &mut CircuitBuilder<GoldilocksField, 2>, 
        board: BoardTargets, 
        piece: PieceStateTargets,
        ledger: LedgerTargets
    ) -> Self{
        GameState { 
            board: board,  
            current_piece: piece, 
            last_action_was_rotation: builder._false(), 
            ledger: ledger}
    }

    fn apply_movement(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, action_type: ActionTargets) -> Self{
        let action = action_type.action;
        let current_piece = self.current_piece;
        let board = self.board;
        let mut last_action_rotate = self.last_action_was_rotation;

        let zero = builder.zero();
        let one = builder.one();
        let two = builder.constant(GoldilocksField::from_canonical_usize(2));
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));
        let four = builder.constant(GoldilocksField::from_canonical_usize(4));
        let five = builder.constant(GoldilocksField::from_canonical_usize(5));

        let is_left = builder.is_equal(zero,action);
        let is_right = builder.is_equal(action, one);
        let is_cw = builder.is_equal(action, two);
        let is_ccw = builder.is_equal(action, three);
        let is_sd = builder.is_equal(action, four);
        let is_hd = builder.is_equal(action, five);

        let (left_piece, left_ok) = current_piece.shift_left(builder, board);
        let (right_piece, right_ok) = current_piece.shift_right(builder, board);
        let (sd_piece,sd_ok) = current_piece.soft_drop(builder, board);
        let (cw_piece, cw_ok) = current_piece.rotateCW(builder, board);
        let (ccw_piece, ccw_ok) = current_piece.rotateCCW(builder, board);
        let (hd_piece, hd_ok) = current_piece.hard_drop(builder, board);

        let moved_left = builder.and(is_left,left_ok);
        let didnt_left = builder.not(moved_left);
        let moved_right = builder.and(is_right, right_ok);
        let didnt_right = builder.not(moved_right);
        let rotated_cw = builder.and(is_cw, cw_ok);
        let rotated_ccw = builder.and(is_ccw, ccw_ok);
        let moved_sd = builder.and(is_sd, sd_ok);
        let didnt_sd = builder.not(moved_sd);
        let moved_hd = builder.and(is_hd, hd_ok);
        let didnt_hd = builder.not(moved_hd);

        last_action_rotate = builder.and(didnt_left, last_action_rotate);
        last_action_rotate = builder.and(didnt_right, last_action_rotate);
        last_action_rotate = builder.and(didnt_sd, last_action_rotate);
        last_action_rotate = builder.and(didnt_hd, last_action_rotate);
        last_action_rotate = builder.or(rotated_cw, last_action_rotate);
        last_action_rotate = builder.or(rotated_ccw, last_action_rotate);

        let mut adjusted_piece = current_piece;
        adjusted_piece = select_piece_state(builder, is_left, left_piece, adjusted_piece);
        adjusted_piece = select_piece_state(builder, is_right, right_piece, adjusted_piece);
        adjusted_piece = select_piece_state(builder, is_cw, cw_piece, adjusted_piece);
        adjusted_piece = select_piece_state(builder, is_ccw, ccw_piece, adjusted_piece);
        adjusted_piece = select_piece_state(builder, is_sd, sd_piece, adjusted_piece);
        adjusted_piece = select_piece_state(builder, is_hd, hd_piece, adjusted_piece);

        GameState { 
            board: board, 
            current_piece: adjusted_piece, 
            last_action_was_rotation: last_action_rotate, 
            ledger: self.ledger
        }
    }


    fn lock_piece(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> GameState {
        let board = self.board;
        let (adjusted_piece, hd_true) = self.current_piece.hard_drop(builder, board);
        let no_hd = builder.not(hd_true);
        let last_action_rotate =builder.and(self.last_action_was_rotation, no_hd);
        let old_ledger = self.ledger;

        let one = builder.one();
        let two = builder.constant(GoldilocksField::from_canonical_usize(2));
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));
        let four = builder.constant(GoldilocksField::from_canonical_usize(4));

        let placed_board = board.place(builder, adjusted_piece);
        let (cleared_board, lines_cleared) = placed_board.clear_lines(builder);

        let is_pc = cleared_board.check_empty(builder);
        let is_single = builder.is_equal(lines_cleared, one);
        let is_double = builder.is_equal(lines_cleared, two);
        let is_triple = builder.is_equal(lines_cleared, three);
        let is_tetris = builder.is_equal(lines_cleared, four);
        let three_corners = adjusted_piece.three_corners(builder, board);
        let is_tspin = builder.and(three_corners, last_action_rotate);
        let is_tss = builder.and(is_tspin, is_single);
        let is_tsd = builder.and(is_tspin, is_double);
        let is_tst = builder.and(is_tspin, is_triple);

        let new_ledger = LedgerTargets{
            tss: builder.add(old_ledger.tss, is_tss.target),
            tsd: builder.add(old_ledger.tsd, is_tsd.target),
            tst: builder.add(old_ledger.tst, is_tst.target),
            tetris: builder.add(old_ledger.tetris, is_tetris.target),
            pc: builder.add(old_ledger.pc, is_pc.target),
        };

        GameState { 
            board: cleared_board, 
            current_piece: adjusted_piece, 
            last_action_was_rotation: builder._false(), 
            ledger: new_ledger 
        }
    }

}

#[derive(Debug, Clone, Copy)]
struct ActionTargets{
    action: Target // left right cw ccw sd hd
}


#[derive(Debug, Clone)]
struct TetrisCircuit {
    initial_board: BoardTargets, 
    pieces: Vec<Target>,          
    actions: Vec<Vec<ActionTargets>>,      
    ledger: LedgerTargets,
}

#[derive(Debug, Clone, Copy)]
struct BoardTargets{
    cells: [[BoolTarget;10];25]
}

impl BoardTargets{
    fn empty(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        Self{ cells: [[builder._false();10];25] }
    }

    fn no_collision(&self, builder:&mut CircuitBuilder<GoldilocksField, 2>, shape: [[Target;2];4], row: Target, col: Target) -> BoolTarget {
        let mut any_collision = builder._false();
        let cells = self.cells;

        for block in 0..4 {
            let piece_row = builder.add(row,shape[block][1]); // these are flipped because 
            let piece_col = builder.add(col,shape[block][0]); // in the table they're x and y

            let in_row = in_range(builder, piece_row, 0, 25);
            let not_row = builder.not(in_row);
            let in_col = in_range(builder, piece_col, 0, 10);
            let not_col = builder.not(in_col);
            any_collision = builder.or(any_collision, not_row);
            any_collision = builder.or(any_collision, not_col);

            for board_row in 0..25 {
                for board_col in 0..10 {
                    let board_rt = builder.constant(GoldilocksField::from_canonical_usize(board_row));
                    let board_ct = builder.constant(GoldilocksField::from_canonical_usize(board_col));
                    let is_row = builder.is_equal(piece_row, board_rt);
                    let is_col = builder.is_equal(piece_col, board_ct);
                    let is_piece = builder.and(is_col, is_row);

                    let collision = builder.and(is_piece, cells[board_row][board_col]);
                    any_collision = builder.or(any_collision, collision);
                }
            }
        }
        builder.not(any_collision)
    }

    fn place(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, piece_state: PieceStateTargets) -> BoardTargets {
        let mut cells = self.cells;
        let shape = piece_state.get_shape(builder);
        for block in 0..4 {
            let piece_row = builder.add(piece_state.row,shape[block][1]);
            let piece_col = builder.add(piece_state.col,shape[block][0]);

            for board_row in 0..25 {
                for board_col in 0..10 {
                    let board_rt = builder.constant(GoldilocksField::from_canonical_usize(board_row));
                    let board_ct = builder.constant(GoldilocksField::from_canonical_usize(board_col));
                    let is_row = builder.is_equal(piece_row, board_rt);
                    let is_col = builder.is_equal(piece_col, board_ct);
                    let is_piece = builder.and(is_col, is_row);

                    cells[board_row][board_col] = builder.or(is_piece, cells[board_row][board_col]);
                }
            }
        }
        BoardTargets { cells }
    }

    fn full_rows(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> [BoolTarget;25] {
        let cells = self.cells;
        let mut counter = [builder._false();25];
        for board_row in 0..25{
            let mut full = builder._true();
                for board_col in 0..10{
                    full = builder.and(full, cells[board_row][board_col]);
                }
            counter[board_row] = full;
        }
        counter
    }


    fn clear_lines(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> (BoardTargets, Target)  {
        let old_board = self.cells;
        let mut new_board = [[builder._false();10];25];
        let full_rows =self.full_rows(builder);
        let mut cleared_rows = builder.zero();

        for old_row in (0..25).rev() {
            let row_full = full_rows[old_row];
            let row_not_full = builder.not(row_full);
            cleared_rows = builder.add(cleared_rows,row_full.target);

            let old_yt = builder.constant(GoldilocksField::from_canonical_usize(old_row));
            let shifted_yt = builder.add(cleared_rows,old_yt);

            for new_row in (0..25).rev() {
                let new_yt = builder.constant(GoldilocksField::from_canonical_usize(new_row));
                let write_here = builder.is_equal(shifted_yt, new_yt);
                let copy_this_row = builder.and(write_here, row_not_full);

                for col in 0..10 {
                    let selected = builder.select(
                        copy_this_row, 
                        old_board[old_row][col].target, 
                        new_board[new_row][col].target
                    );
                    new_board[new_row][col] = BoolTarget::new_unsafe(selected);
                }
            }

        }
        (BoardTargets{ cells: new_board }, cleared_rows)
    }

    fn check_empty(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> BoolTarget {
        let mut filled = builder._false();
        for row in 0..25 {
            for col in 0..10{
                filled = builder.or(self.cells[row][col], filled);
            }
        }
        builder.not(filled)
    }


}

const PIECE_SHAPE: [[[[u32; 2]; 4]; 4]; 7] = [
    [ // I piece
        [[0, 1], [1, 1], [2, 1], [3, 1]],
        [[2, 0], [2, 1], [2, 2], [2, 3]],
        [[0, 2], [1, 2], [2, 2], [3, 2]],
        [[1, 0], [1, 1], [1, 2], [1, 3]],
    ],

    [ // O piece
        [[0, 0], [1, 0], [0, 1], [1, 1]],
        [[0, 0], [1, 0], [0, 1], [1, 1]],
        [[0, 0], [1, 0], [0, 1], [1, 1]],
        [[0, 0], [1, 0], [0, 1], [1, 1]],
    ],

    [ // T piece
        [[1, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [2, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [1, 2]],
        [[1, 0], [0, 1], [1, 1], [1, 2]],
    ],

    [ // S piece
        [[1, 0], [2, 0], [0, 1], [1, 1]],
        [[1, 0], [1, 1], [2, 1], [2, 2]],
        [[1, 1], [2, 1], [0, 2], [1, 2]],
        [[0, 0], [0, 1], [1, 1], [1, 2]],
    ],

    [ // Z piece
        [[0, 0], [1, 0], [1, 1], [2, 1]],
        [[2, 0], [1, 1], [2, 1], [1, 2]],
        [[0, 1], [1, 1], [1, 2], [2, 2]],
        [[1, 0], [0, 1], [1, 1], [0, 2]],
    ],

    [ // L piece
        [[2, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [1, 2], [2, 2]],
        [[0, 1], [1, 1], [2, 1], [0, 2]],
        [[0, 0], [1, 0], [1, 1], [1, 2]],
    ],

    [ // J piece
        [[0, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [2, 2]],
        [[1, 0], [1, 1], [0, 2], [1, 2]],
    ],
];


pub const KICK_TABLES: [[[[i64; 2]; 5]; 8]; 2] = [
    [ // J, L, S, Z, T
        [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
        [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
        [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
        [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
        [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
        [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
        [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
        [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    ],
    [ // I
        [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
        [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
        [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
        [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
        [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
        [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
        [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
        [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    ],
];


#[derive(Debug, Clone, Copy)]
struct PieceStateTargets{
    piece: Target, //// I O T S Z L J
    rotation: Target,
    row: Target,
    col: Target,
}

impl PieceStateTargets{

    fn spawn(letter: Target, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (Self, BoolTarget) {
        let one = builder.one();
        let seven = builder.constant(GoldilocksField::from_canonical_usize(7));
        let is_seven = builder.is_equal(letter, seven);
        builder.range_check(letter, 3);
        builder.assert_zero(is_seven.target);

        let is_one = builder.is_equal(letter, one);
        let five = builder.constant(GoldilocksField::from_canonical_usize(5));
        let four = builder.constant(GoldilocksField::from_canonical_usize(4));
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));

        let piece_state = PieceStateTargets{ 
            piece: letter, 
            rotation: builder.zero(), 
            row: five,
            col: builder.select(is_one,four, three)
        };

        let piece_shape = piece_state.get_shape(builder);
        let game_okay = board.no_collision(builder, piece_shape, piece_state.row, piece_state.col);

        (piece_state, game_okay)
    }

    fn get_shape(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> [[Target;2];4] {
        let piece = self.piece;
        let rotation = self.rotation;
        let zero = builder.zero();
        let mut coords = [[zero;2];4];

        builder.range_check(piece, 3);
        builder.range_check(rotation, 2);


        for p_id in 0..7 {
            let letter = builder.constant(GoldilocksField::from_canonical_usize(p_id));
            let piece_matches = builder.is_equal(piece, letter);

            for r_id in 0..4 {
                let direction = builder.constant(GoldilocksField::from_canonical_usize(r_id));
                let rotation_matches = builder.is_equal(rotation, direction);

                let selected = builder.and(piece_matches,rotation_matches);

                for block in 0..4 {
                    for axis in 0..2 {
                        let value = PIECE_SHAPE[p_id][r_id][block][axis];
                        let value_t = builder.constant(GoldilocksField::from_canonical_u32(value));

                        coords[block][axis] = builder.select(selected, value_t, coords[block][axis]);
                    }
                }
            }
        }
        coords
    }

    fn shift_right(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let one = builder.constant(GoldilocksField::from_canonical_usize(1));
        let new_col = builder.add(self.col, one);
        let shape = self.get_shape(builder);
        let shiftable = board.no_collision(builder, shape, self.row, new_col);

        (PieceStateTargets { 
            piece: self.piece, 
            rotation: self.rotation, 
            row: self.row, 
            col: builder.add(self.col, shiftable.target) 
        },
        shiftable)
    }

    fn shift_left(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let one = builder.constant(GoldilocksField::from_canonical_usize(1));
        let new_col = builder.sub(self.col, one);
        let shape = self.get_shape(builder);
        let shiftable = board.no_collision(builder, shape, self.row, new_col);

        (PieceStateTargets { 
            piece: self.piece, 
            rotation: self.rotation, 
            row: self.row, 
            col: builder.sub(self.col, shiftable.target) 
        },
        shiftable)
    }

    fn soft_drop(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let one = builder.constant(GoldilocksField::from_canonical_usize(1));
        let new_row = builder.add(self.row, one);
        let shape = self.get_shape(builder);
        let shiftable = board.no_collision(builder, shape, new_row, self.col);

        (
            PieceStateTargets { 
                piece: self.piece, 
                rotation: self.rotation, 
                row: builder.add(self.row, shiftable.target),
                col: self.col
            },
            shiftable
        )
    }

    fn hard_drop(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let mut total_shifted = builder._false();
        let mut piece = *self;
        
        for _ in 0..25 {
            let (next_state, shifted) = piece.soft_drop(builder, board);
            total_shifted = builder.or(total_shifted,shifted);
            piece = next_state;
        }

        (piece, total_shifted)
    }

    fn try_kick(
        &self,
        builder: &mut CircuitBuilder<GoldilocksField, 2>,
        transition: usize,
        test: usize,
    ) -> [Target; 2] {
        let piece = self.piece;
        let one = builder.one();
        let zero = builder.zero();
        let is_i = builder.is_equal(piece, zero);
        let is_o = builder.is_equal(piece, one);
        let not_o = builder.not(is_o);

        let jlszt_dx = builder.constant(GoldilocksField::from_canonical_i64(
            KICK_TABLES[0][transition][test][0],
        ));
        let jlszt_dy = builder.constant(GoldilocksField::from_canonical_i64(
            KICK_TABLES[0][transition][test][1],
        ));

        let i_dx = builder.constant(GoldilocksField::from_canonical_i64(
            KICK_TABLES[1][transition][test][0],
        ));
        let i_dy = builder.constant(GoldilocksField::from_canonical_i64(
            KICK_TABLES[1][transition][test][1],
        ));

        let dx = builder.select(is_i, i_dx, jlszt_dx);
        let dy = builder.select(is_i, i_dy, jlszt_dy);

        [builder.mul(dx, not_o.target), builder.mul(dy, not_o.target)]
    }

    fn rotateCW(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let mut found = builder._false();
        let mut final_row = self.row;
        let mut final_col = self.col;

        let initial_rotation = self.rotation;
        let zero = builder.zero();
        let one = builder.one();
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));
        let is_three = builder.is_equal(initial_rotation, three);
        let addition = builder.add(initial_rotation, one);
        let target_rotation = builder.select(is_three, zero, addition);
        let new_shape = PieceStateTargets {
            piece: self.piece, 
            rotation: target_rotation, 
            row: self.row, 
            col: self.col
        };
        let shape_coord = new_shape.get_shape(builder);

        for orientation in 0..4{
            let orientation_t = builder.constant(GoldilocksField::from_canonical_usize(orientation));
            let this_rotate = builder.is_equal(initial_rotation, orientation_t);

            for kick in 0..5{
                let transition = orientation * 2 + 1;
                let [dx, dy] = self.try_kick(builder, transition, kick);
                let try_row = builder.sub(self.row, dy);
                let try_col = builder.add(self.col, dx);

                let works = board.no_collision(builder, shape_coord, try_row, try_col);
                let correct_kick = builder.and(works, this_rotate);
                let not_found = builder.not(found);
                let update_pos = builder.and(correct_kick, not_found);

                final_row = builder.select(update_pos, try_row, final_row);
                final_col = builder.select(update_pos, try_col, final_col);
                found = builder.or(found,update_pos);
            }
        }
        let final_rotation = builder.select(found, target_rotation, initial_rotation);
        (PieceStateTargets { piece: self.piece, rotation: final_rotation, row: final_row, col: final_col }, found)
    }

    fn rotateCCW(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> (PieceStateTargets, BoolTarget) {
        let mut found = builder._false();
        let mut final_row = self.row;
        let mut final_col = self.col;

        let initial_rotation = self.rotation;
        let zero = builder.zero();
        let one = builder.one();
        let is_zero = builder.is_equal(initial_rotation, zero);
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));
        let subtraction = builder.sub(initial_rotation, one);
        let target_rotation = builder.select(is_zero, three, subtraction);
        let new_shape = PieceStateTargets {
            piece: self.piece, 
            rotation: target_rotation, 
            row: self.row, 
            col: self.col
        };
        let shape_coord = new_shape.get_shape(builder);

        for orientation in 0..4{
            let orientation_t = builder.constant(GoldilocksField::from_canonical_usize(orientation));
            let this_rotate = builder.is_equal(initial_rotation, orientation_t);

            for kick in 0..5{
                let transition = orientation * 2;
                let [dx, dy] = self.try_kick(builder, transition, kick);
                let try_row = builder.sub(self.row, dy);
                let try_col = builder.add(self.col, dx);

                let works = board.no_collision(builder, shape_coord, try_row, try_col);
                let correct_kick = builder.and(works, this_rotate);
                let not_found = builder.not(found);
                let update_pos = builder.and(correct_kick, not_found);

                final_row = builder.select(update_pos, try_row, final_row);
                final_col = builder.select(update_pos, try_col, final_col);
                found = builder.or(found,update_pos);
            }
        }
        let final_rotation = builder.select(found, target_rotation, initial_rotation);
        (PieceStateTargets { piece: self.piece, rotation: final_rotation, row: final_row, col: final_col }, found)
    }

    fn three_corners(&self, builder:&mut CircuitBuilder<GoldilocksField, 2>, board: BoardTargets) -> BoolTarget {
        let mut num_collisions = builder.zero();
        let cells = board.cells;
        let row = self.row;
        let col = self.col;
        
        let zero = builder.zero();
        let two = builder.constant(GoldilocksField::from_canonical_usize(2));
        let shape = [[zero,zero],[two,zero],[zero,two],[two,two]];
        for block in 0..4 {
            let piece_row = builder.add(row,shape[block][1]);
            let piece_col = builder.add(col,shape[block][0]);

            let in_row =in_range(builder, piece_row, 0, 25);
            let not_row = builder.not(in_row);
            let in_col = in_range(builder, piece_col, 0, 10);
            let not_col = builder.not(in_col);
            let corner_blocked = builder.or(not_col,not_row);
            num_collisions = builder.add(num_collisions, corner_blocked.target);

            for board_row in 0..25 {
                for board_col in 0..10 {
                    let board_rt = builder.constant(GoldilocksField::from_canonical_usize(board_row));
                    let board_ct = builder.constant(GoldilocksField::from_canonical_usize(board_col));
                    let is_row = builder.is_equal(piece_row, board_rt);
                    let is_col = builder.is_equal(piece_col, board_ct);
                    let is_piece = builder.and(is_col, is_row);

                    let collision = builder.and(is_piece, cells[board_row][board_col]);
                    num_collisions = builder.add(num_collisions, collision.target);
                    
                }
            }
        }
        in_range(builder, num_collisions, 3, 5)
    }


}

#[derive(Debug, Clone, Copy)]
struct LedgerTargets{
    tss: Target,
    tsd: Target,
    tst: Target,
    tetris: Target,
    pc: Target,
}

impl LedgerTargets{
    fn empty(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self{
        Self { 
            tss: builder.zero(), 
            tsd: builder.zero(), 
            tst: builder.zero(), 
            tetris: builder.zero(), 
            pc: builder.zero() 
        }
    }
}

#[test]
fn test_simulate_one_o_piece_empty_board() {
    use plonky2::iop::witness::PartialWitness;
    use plonky2::plonk::circuit_builder::CircuitBuilder;
    use plonky2::plonk::circuit_data::CircuitConfig;

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(config);

    let board = BoardTargets::empty(&mut builder);

    let o_piece = builder.constant(GoldilocksField::from_canonical_usize(1));
    let queue = vec![o_piece];

    let actions: Vec<Vec<ActionTargets>> = vec![vec![]];

    let ledger = simulate(&mut builder, board, queue, actions);

    builder.assert_zero(ledger.tss);
    builder.assert_zero(ledger.tsd);
    builder.assert_zero(ledger.tst);
    builder.assert_zero(ledger.tetris);
    builder.assert_zero(ledger.pc);

    let data = builder.build::<plonky2::plonk::config::PoseidonGoldilocksConfig>();
    let pw = PartialWitness::new();

    let proof = data.prove(pw).unwrap();
    data.verify(proof).unwrap();
}

