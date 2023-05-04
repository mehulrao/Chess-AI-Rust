use chess::{CacheTable, ChessMove, Square, Board, MoveGen, Piece, Rank, File};
use crate::{evaluation, searcher};

use super::entry::Entry;


const MAX_MOVE_CNT: usize = 100;
const SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN: u32 = 350;
const CAP_PIECE_VALUE_MULTIPLIER: u32 = 10;

pub fn order_moves(board: &Board, tt: &CacheTable<Entry>, move_list: MoveGen, use_tt: bool) -> Vec<ChessMove> {
    let mut move_list = move_list.into_iter().collect::<Vec<ChessMove>>();
    let mut hash_move;
    let mut move_scores: [u32; 100] = [0; MAX_MOVE_CNT];
    for (count, _move) in move_list.clone().into_iter().enumerate() {
        let mut score = 0;
        let move_piece_type = board.piece_on(_move.get_source()).unwrap();
        let capture_piece_type = board.piece_on(_move.get_dest());
        if capture_piece_type.is_some() {
            score = CAP_PIECE_VALUE_MULTIPLIER * get_piece_value(capture_piece_type.unwrap()) - get_piece_value(move_piece_type);
        }
        if move_piece_type == Piece::Pawn {
            if _move.get_promotion().is_some() {
                score += get_piece_value(_move.get_promotion().unwrap());
            }
        } else {
            // check if destination square is controlled by an opponent pawn
            let dest_square = _move.get_dest();
            let dest_square_file = dest_square.get_file().to_index();
            let dest_square_rank = dest_square.get_rank().to_index();
            let opponent_pawn_square_1: Square;
            let opponent_pawn_square_2: Square;
            if board.side_to_move() == chess::Color::White {
                opponent_pawn_square_1 = Square::make_square(Rank::from_index(dest_square_rank + 1), File::from_index(dest_square_file + 1));
                opponent_pawn_square_2 = Square::make_square(Rank::from_index(dest_square_rank + 1), File::from_index(dest_square_file - 1));
            } else {
                opponent_pawn_square_1 = Square::make_square(Rank::from_index(dest_square_rank - 1), File::from_index(dest_square_file + 1));
                opponent_pawn_square_2 = Square::make_square(Rank::from_index(dest_square_rank - 1), File::from_index(dest_square_file - 1));
            }
            if board.piece_on(opponent_pawn_square_1).is_some() && board.piece_on(opponent_pawn_square_1).unwrap() == Piece::Pawn {
                score -= SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN;
            } else if board.piece_on(opponent_pawn_square_2).is_some() && board.piece_on(opponent_pawn_square_2).unwrap() == Piece::Pawn {
                score -= SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN;
            }
        }
        if use_tt {
            hash_move = searcher::get_stored_move(tt, board.get_hash());
            if hash_move.is_some() && hash_move.unwrap() == _move {
                score += 10000;
            }
        }
        move_scores[count] = score;
    }
    return sort(&mut move_scores, &mut move_list);
}

fn get_piece_value(piece_type: chess::Piece) -> u32 {
    match piece_type {
        chess::Piece::Pawn => evaluation::PAWN_VALUE,
        chess::Piece::Knight => evaluation::KNIGHT_VALUE,
        chess::Piece::Bishop => evaluation::BISHOP_VALUE,
        chess::Piece::Rook => evaluation::ROOK_VALUE,
        chess::Piece::Queen => evaluation::QUEEN_VALUE,
        _ => 0,
    }
}

fn sort(move_scores: &mut [u32; MAX_MOVE_CNT], move_list: &mut Vec<ChessMove>) -> Vec<ChessMove> {
    let mut move_list = move_list.to_vec();
    for i in 0..move_list.len() {
        for j in (i + 1)..0 {
            let swap_index = j - 1;
            if move_scores[swap_index] > move_scores[j] {
                let temp_score = move_scores[swap_index];
                move_scores[swap_index] = move_scores[j];
                move_scores[j] = temp_score;
                let temp_move = move_list[swap_index];
                move_list[swap_index] = move_list[j];
                move_list[j] = temp_move;
            }
        }
    }
    return move_list
}