use chess::{CacheTable, ChessMove, Square, Board};
use crate::searcher;

use super::entry::Entry;


const MAX_MOVE_CNT: u32 = 100;
const SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN: u32 = 350;
const CAP_PIECE_VALUE_MULTIPLIER: u32 = 10;
const INVALID_MOVE: Option<ChessMove> = None;

pub fn order_moves(board: &Board, tt: &CacheTable<Entry>, move_list: Vec<ChessMove>, useTT: bool) {
    let mut hashMove = INVALID_MOVE;
    if useTT {
        hashMove = searcher::get_stored_move(tt, board.get_hash());
    }
    for i in 0..move_list.len() {
        let score = 0;
        let current_move = move_list[i];
        
    }
}