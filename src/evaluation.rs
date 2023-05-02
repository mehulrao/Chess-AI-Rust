use std::ops::BitAnd;

use chess::{Board, Color, Piece, Square, ALL_SQUARES};

const PAWNS: [i32; 64] = [
                        0,  0,  0,  0,  0,  0,  0,  0,
                        50, 50, 50, 50, 50, 50, 50, 50,
                        10, 10, 20, 30, 30, 20, 10, 10,
                        5,  5, 10, 25, 25, 10,  5,  5,
                        0,  0,  0, 20, 20,  0,  0,  0,
                        5, -5,-10,  0,  0,-10, -5,  5,
                        5, 10, 10,-20,-20, 10, 10,  5,
                        0,  0,  0,  0,  0,  0,  0,  0];

const KNIGHTS: [i32; 64] = [
                        -50,-40,-30,-30,-30,-30,-40,-50,
                        -40,-20,  0,  0,  0,  0,-20,-40,
                        -30,  0, 10, 15, 15, 10,  0,-30,
                        -30,  5, 15, 20, 20, 15,  5,-30,
                        -30,  0, 15, 20, 20, 15,  0,-30,
                        -30,  5, 10, 15, 15, 10,  5,-30,
                        -40,-20,  0,  5,  5,  0,-20,-40,
                        -50,-40,-30,-30,-30,-30,-40,-50];

const BISHOPS: [i32; 64] = [
                        -20,-10,-10,-10,-10,-10,-10,-20,
                        -10,  0,  0,  0,  0,  0,  0,-10,
                        -10,  0,  5, 10, 10,  5,  0,-10,
                        -10,  5,  5, 10, 10,  5,  5,-10,
                        -10,  0, 10, 10, 10, 10,  0,-10,
                        -10, 10, 10, 10, 10, 10, 10,-10,
                        -10,  5,  0,  0,  0,  0,  5,-10,
                        -20,-10,-10,-10,-10,-10,-10,-20];

const ROOKS: [i32; 64] = [
                        0,  0,  0,  0,  0,  0,  0,  0,
                        5, 10, 10, 10, 10, 10, 10,  5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        -5,  0,  0,  0,  0,  0,  0, -5,
                        0,  0,  0,  5,  5,  0,  0,  0];

const QUEENS: [i32; 64] = [
                        -20,-10,-10, -5, -5,-10,-10,-20,
                        -10,  0,  0,  0,  0,  0,  0,-10,
                        -10,  0,  5,  5,  5,  5,  0,-10,
                        -5,  0,  5,  5,  5,  5,  0, -5,
                        0,  0,  5,  5,  5,  5,  0, -5,
                        -10,  5,  5,  5,  5,  5,  0,-10,
                        -10,  0,  5,  0,  0,  0,  0,-10,
                        -20,-10,-10, -5, -5,-10,-10,-20];

const KING_MIDDLE: [i32; 64] = [
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

pub fn evaluate(board: &Board) -> i32 {
    let perspective;
    let mut white_eval = count_material(Color::White, board);
    white_eval += evaluate_tables(Color::White, board);
    let mut black_eval = count_material(Color::Black, board);
    black_eval += evaluate_tables(Color::Black, board);
    match board.side_to_move() {
        Color::White => perspective = 1,
        Color::Black => perspective = -1,
    }
    if get_checkers(Color::White, board){
        black_eval -= 25;
    }
    if get_checkers(Color::Black, board) {
        white_eval -= 25;
    }
    let eval = (white_eval - black_eval) * perspective;
    eval
}

fn count_material(color: Color, board: &Board) -> i32 {
    let mut material = 0;
    let pieces = board.color_combined(color);
    material += board.pieces(Piece::Pawn).bitand(pieces).popcnt() * PAWN_VALUE;
    material += board.pieces(Piece::Knight).bitand(pieces).popcnt() * KNIGHT_VALUE;
    material += board.pieces(Piece::Bishop).bitand(pieces).popcnt() * BISHOP_VALUE;
    material += board.pieces(Piece::Rook).bitand(pieces).popcnt() * ROOK_VALUE;
    material += board.pieces(Piece::Queen).bitand(pieces).popcnt() * QUEEN_VALUE;
    material as i32
}

fn get_checkers(color: Color, board: &Board) -> bool {
    let checkers = board.checkers();
    let pieces = board.color_combined(color);
    if checkers.bitand(pieces).popcnt() > 0 {
        true
    } else {
        false
    }
}

fn evaluate_tables(color: Color, board: &Board) -> i32 {
    let mut score = 0;
    for sq in ALL_SQUARES.iter() {
        if board.piece_on(*sq).is_some() && board.color_on(*sq).unwrap() == color {
            let index = if color == Color::Black {sq.to_index()} else {63 - sq.to_index()};
            match board.piece_on(*sq).unwrap() {
                Piece::Pawn => {
                    score += PAWNS[index];
                },
                Piece::Knight => {
                    score += KNIGHTS[index];
                },
                Piece::Bishop => {
                    score += BISHOPS[index];
                },
                Piece::Rook => {
                    score += ROOKS[index];
                },
                Piece::Queen => {
                    score += QUEENS[index];
                },
                Piece::King => {
                    if board.pieces(Piece::Pawn).bitand(board.color_combined(color)).popcnt() < 2 {
                        score += KING_MIDDLE[index];
                    }
                },
            }
        }
    }
    score
}

