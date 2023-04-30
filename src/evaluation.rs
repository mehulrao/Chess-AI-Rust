use std::ops::BitAnd;

use chess::{Board, Color, Piece};

const PAWNS: [i8; 64] = [
                        0,  0,  0,  0,  0,  0,  0,  0,
                        50, 50, 50, 50, 50, 50, 50, 50,
                        10, 10, 20, 30, 30, 20, 10, 10,
                        5,  5, 10, 25, 25, 10,  5,  5,
                        0,  0,  0, 20, 20,  0,  0,  0,
                        5, -5,-10,  0,  0,-10, -5,  5,
                        5, 10, 10,-20,-20, 10, 10,  5,
                        0,  0,  0,  0,  0,  0,  0,  0];

const KNIGHTS: [i8; 64] = [
                        -50,-40,-30,-30,-30,-30,-40,-50,
                        -40,-20,  0,  0,  0,  0,-20,-40,
                        -30,  0, 10, 15, 15, 10,  0,-30,
                        -30,  5, 15, 20, 20, 15,  5,-30,
                        -30,  0, 15, 20, 20, 15,  0,-30,
                        -30,  5, 10, 15, 15, 10,  5,-30,
                        -40,-20,  0,  5,  5,  0,-20,-40,
                        -50,-40,-30,-30,-30,-30,-40,-50];

const BISHOPS: [i8; 64] = [
                        -20,-10,-10,-10,-10,-10,-10,-20,
                        -10,  0,  0,  0,  0,  0,  0,-10,
                        -10,  0,  5, 10, 10,  5,  0,-10,
                        -10,  5,  5, 10, 10,  5,  5,-10,
                        -10,  0, 10, 10, 10, 10,  0,-10,
                        -10, 10, 10, 10, 10, 10, 10,-10,
                        -10,  5,  0,  0,  0,  0,  5,-10,
                        -20,-10,-10,-10,-10,-10,-10,-20];

const ROOKS: [i8; 64] = [
                        0,  0,  0,  0,  0,  0,  0,  0,
                        5, 10, 10, 10, 10, 10, 10,  5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        0,  0,  0,  5,  5,  0,  0,  0];

const QUEENS: [i8; 64] = [
                        -20,-10,-10, -5, -5,-10,-10,-20,
                        -10,  0,  0,  0,  0,  0,  0,-10,
                        -10,  0,  5,  5,  5,  5,  0,-10,
                        -5,  0,  5,  5,  5,  5,  0, -5,
                        0,  0,  5,  5,  5,  5,  0, -5,
                        -10,  5,  5,  5,  5,  5,  0,-10,
                        -10,  0,  5,  0,  0,  0,  0,-10,
                        -20,-10,-10, -5, -5,-10,-10,-20];

const KING_MIDDLE: [i8; 64] = [
                            -30,-40,-40,-50,-50,-40,-40,-30,
                            -30,-40,-40,-50,-50,-40,-40,-30,
                            -30,-40,-40,-50,-50,-40,-40,-30,
                            -30,-40,-40,-50,-50,-40,-40,-30,
                            -20,-30,-30,-40,-40,-30,-30,-20,
                            -10,-20,-20,-20,-20,-20,-20,-10,
                            20, 20,  0,  0,  0,  0, 20, 20,
                            20, 30, 10,  0,  0, 10, 30, 20
                        ];

const PAWN_VALUE: u32 = 100;
const KNIGHT_VALUE: u32 = 320;
const BISHOP_VALUE: u32 = 330;
const ROOK_VALUE: u32 = 500;
const QUEEN_VALUE: u32 = 900;

const ENDGAME_MATERIAL_START: u32 = ROOK_VALUE * 2 + BISHOP_VALUE + KNIGHT_VALUE;

pub struct Evaluation {
    board: Board,
}

impl Evaluation {
    pub fn new(board: Board) -> Evaluation {
        let mut eval = Evaluation {
            board,
        };
        eval
    }
    pub fn evaluate(&self) -> i32 {
        let mut perspective = 0;
        let mut white_eval = self.count_material(Color::White);
        let mut black_eval = self.count_material(Color::Black);
        match self.board.side_to_move() {
            Color::White => perspective = 0,
            Color::Black => perspective = 1,
        }
        if self.get_num_checkers(Color::White){ 
            println!("removing black eval due to check");
            black_eval -= 25;
        }
        if self.get_num_checkers(Color::Black) {
            println!("removing white eval due to check");
            white_eval -= 25;
        }
        let eval = (white_eval - black_eval) * perspective;
        eval
    }

    fn count_material(&self, color: Color) -> i32 {
        let mut material = 0;
        let pieces = self.board.color_combined(color);
        material += self.board.pieces(Piece::Pawn).bitand(pieces).popcnt() * PAWN_VALUE;
        material += self.board.pieces(Piece::Knight).bitand(pieces).popcnt() * KNIGHT_VALUE;
        material += self.board.pieces(Piece::Bishop).bitand(pieces).popcnt() * BISHOP_VALUE;
        material += self.board.pieces(Piece::Rook).bitand(pieces).popcnt() * ROOK_VALUE;
        material += self.board.pieces(Piece::Queen).bitand(pieces).popcnt() * QUEEN_VALUE;
        material as i32
    }
    
    fn get_num_checkers(&self, color: Color) -> bool {
        let checkers = self.board.checkers();
        let pieces = self.board.color_combined(color);
        if checkers.bitand(pieces).popcnt() > 0 {
            true
        } else {
            false
        }
    }
}
