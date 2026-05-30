use plonky2::field::types::{Field, Field64};
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::field::goldilocks_field::GoldilocksField;

const MAX_PIECES: usize = 30;
const MAX_ACTIONS: usize = 90;

#[derive(Clone, Debug)]
struct TetrisCircuit {
    initial_board: BoardTargets, 
    pieces: Vec<Target>,          
    actions: Vec<Vec<ActionTargets>>,      
    ledger: LedgerTargets,
}

struct BoardTargets{
    cells: Vec<BoolTarget>
}

impl BoardTargets{
    fn new(builder: &mut CircuitBuilder<GoldilocksField, 2>) -> Self {
        let mut cells = Vec::with_capacity(250);
        for _ in 0..250 {
            let cell = builder.add_virtual_bool_target_safe();
            builder.assert_zero(cell.target);
            cells.push(cell);
        }
        BoardTargets{cells}
    }

    fn get_index(row: usize, col: usize) -> usize {
        row * 10 + col
    }

    fn no_collision(&self, builder:&mut CircuitBuilder<GoldilocksField, 2>, shape: [[Target;2];4], row: Target, col: Target) -> BoolTarget {
        for p_row in 0..4{
            let p_row = builder.constant(GoldilocksField::from_canonical_usize(p_row));
            for p_col in 0..4{

            }
        }
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


struct PieceStateTargets{
    piece: Target, //// I O T S Z L J
    rotation: Target,
    row: Target,
    col: Target,
}

impl PieceStateTargets{
    fn get_shape(&self, builder: &mut CircuitBuilder<GoldilocksField, 2>) -> BoolTarget {
        let piece = self.piece;
        let rotation = self.rotation;
        let zero = builder.zero();
        let mut coords = [[zero;2];4];
        let seven = builder.constant(GoldilocksField::from_canonical_usize(7));
        let is_seven = builder.is_equal(piece, seven);
        let three = builder.constant(GoldilocksField::from_canonical_usize(3));
        builder.range_check(piece, 3);
        builder.assert_zero(is_seven.target);
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
}

struct LedgerTargets{
    tss: Target,
    tsd: Target,
    tst: Target,
    tetris: Target,
    pc: Target,
}

struct ActionTargets{
    action: Target // left right cw ccw sd hd place
}

