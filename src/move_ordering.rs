use chess::{CacheTable, ChessMove, Square};
use super::entry::Entry;


const MAX_MOVE_CNT: u32 = 100;
const SQAURE_CONTOLLED_BY_OPPONENT_PAWN_PEN: u32 = 350;
const CAP_PIECE_VALUE_MULTIPLIER: u32 = 10;
const INVALID_MOVE: Option<ChessMove> = None; 

pub struct MoveOrdering {
    tt: CacheTable<Entry>, 
    move_scores: Vec<u32>,
}

impl MoveOrdering {
    pub fn new(tt: CacheTable<Entry>) -> MoveOrdering {
        MoveOrdering {
            tt,
            move_scores: Vec::new(),
        }
    }
    pub fn order_moves(&self, )
}
