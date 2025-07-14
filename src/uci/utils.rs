use crate::entry::Entry;
use chess::{Board, CacheTable, ChessMove, MoveGen, Square};
use std::str::FromStr;

/// Convert a ChessMove to UCI string format (e.g., "e2e4", "e7e8q")
pub fn move_to_uci_string(chess_move: ChessMove) -> String {
    let promotion = match chess_move.get_promotion() {
        Some(piece) => match piece {
            chess::Piece::Queen => "q",
            chess::Piece::Rook => "r",
            chess::Piece::Bishop => "b",
            chess::Piece::Knight => "n",
            _ => "",
        },
        None => "",
    };
    format!(
        "{}{}{}",
        chess_move.get_source(),
        chess_move.get_dest(),
        promotion
    )
}

/// Parse a UCI move string (e.g., "e2e4") into a ChessMove
pub fn parse_uci_move(move_str: &str, board: &Board) -> Option<ChessMove> {
    if move_str.len() < 4 {
        return None;
    }

    let from_str = &move_str[0..2];
    let to_str = &move_str[2..4];

    let from_square = Square::from_str(from_str).ok()?;
    let to_square = Square::from_str(to_str).ok()?;

    let promotion = if move_str.len() == 5 {
        match move_str.chars().nth(4)? {
            'q' => Some(chess::Piece::Queen),
            'r' => Some(chess::Piece::Rook),
            'b' => Some(chess::Piece::Bishop),
            'n' => Some(chess::Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    Some(ChessMove::new(from_square, to_square, promotion))
}

/// Build a principal variation line by following transposition table entries
pub fn build_pv_line(
    board: &Board,
    first_move: ChessMove,
    tt: &CacheTable<Entry>,
    max_depth: usize,
) -> String {
    let mut pv_moves = Vec::new();
    let mut current_board = *board;
    let mut current_move = first_move;

    // Add the first move
    pv_moves.push(move_to_uci_string(current_move));

    // Try to follow the PV using the transposition table
    for _ in 1..max_depth {
        // Apply the current move
        current_board = current_board.make_move_new(current_move);

        // Look up the best response in the transposition table
        if let Some(stored_move) = crate::searcher::get_stored_move(tt, current_board.get_hash()) {
            // Verify this move is legal in the current position
            let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&current_board).collect();
            if legal_moves.contains(&stored_move) {
                pv_moves.push(move_to_uci_string(stored_move));
                current_move = stored_move;
            } else {
                break; // Invalid move in TT, stop here
            }
        } else {
            break; // No move found in TT, stop here
        }
    }

    pv_moves.join(" ")
}
