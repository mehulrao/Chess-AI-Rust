use chess::{CacheTable, ChessMove, Square, Board, MoveGen, Piece, Rank, File};
use crate::{evaluation, searcher};

use super::entry::Entry;


const MAX_MOVE_CNT: usize = 100;
const SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN: u32 = 350;
const CAP_PIECE_VALUE_MULTIPLIER: u32 = 10;

pub fn order_moves(board: &Board, tt: &CacheTable<Entry>, move_gen: MoveGen, use_tt: bool) -> Vec<ChessMove> {
    // We can directly work with iterators without collecting into a vector.
    let mut moves: Vec<(ChessMove, u32)> = move_gen.map(|m| (m, calculate_move_score(board, &m))).collect();

    // If using transposition table, adjust the score for the move found in it.
    if use_tt {
        if let Some(hash_move) = searcher::get_stored_move(tt, board.get_hash()) {
            for &mut (ref m, ref mut score) in &mut moves {
                if *m == hash_move {
                    *score += 10_000; // Adjust score for hash move.
                    break; // There can only be one hash move, so we can break here.
                }
            }
        }
    }

    // Sort by the computed scores in descending order
    moves.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // We only need the moves, not the scores, so we extract them.
    moves.into_iter().map(|(m, _)| m).collect()
}

fn calculate_move_score(board: &Board, m: &ChessMove) -> u32 {
    let mut score = 0;
    let move_piece_type = board.piece_on(m.get_source()).unwrap();
    let capture_piece_type = board.piece_on(m.get_dest());

    // Captures: Prioritize by MVV/LVA (Most Valuable Victim/Least Valuable Attacker)
    if let Some(piece) = capture_piece_type {
        score += CAP_PIECE_VALUE_MULTIPLIER * get_piece_value(piece) - get_piece_value(move_piece_type);
    }

    // Pawn promotion
    if move_piece_type == Piece::Pawn {
        if let Some(promotion) = m.get_promotion() {
            score += get_piece_value(promotion);
        }
    } else {
        // Penalize moves into squares controlled by opponent pawns.
        score -= penalty_for_pawn_controlled_square(board, m.get_dest(), board.side_to_move());
    }

    score
}

fn penalty_for_pawn_controlled_square(board: &Board, dest_square: Square, side_to_move: chess::Color) -> u32 {
    let dest_square_file = dest_square.get_file().to_index();
    let dest_square_rank = dest_square.get_rank().to_index() as isize; // Cast to isize for arithmetic

    let (pawn_rank_offset, start_file, end_file) = match side_to_move {
        chess::Color::White => (-1, 0, 7),
        chess::Color::Black => (1, 0, 7),
    };

    let mut penalty = 0;

    if dest_square_file > start_file {
        let opponent_pawn_square = Square::make_square(
            Rank::from_index((dest_square_rank + pawn_rank_offset) as usize), // Cast back to usize
            File::from_index(dest_square_file - 1),
        );
        if board.piece_on(opponent_pawn_square) == Some(Piece::Pawn) {
            penalty += SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN;
        }
    }

    if dest_square_file < end_file {
        let opponent_pawn_square = Square::make_square(
            Rank::from_index((dest_square_rank + pawn_rank_offset) as usize), // Cast back to usize
            File::from_index(dest_square_file + 1),
        );
        if board.piece_on(opponent_pawn_square) == Some(Piece::Pawn) {
            penalty += SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN;
        }
    }

    penalty
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
