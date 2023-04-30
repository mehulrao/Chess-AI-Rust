use chess::{ChessMove, Square};
use super::searcher::EvalType;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(PartialOrd)]

pub struct Entry {
    pub value: i32,
    pub move_: ChessMove,
    pub depth: u8,
    pub node_type: EvalType,
}

impl Entry  {
    pub fn new(value: i32, move_: ChessMove, depth: u8, node_type: EvalType) -> Entry {
        Entry {
            value,
            move_,
            depth,
            node_type,
        }
    }
    pub fn new_default() -> Entry {
        Entry {
            value: 0,
            move_: ChessMove::new(Square::default(), Square::default(), None),
            depth: 0,
            node_type: EvalType::Exact,
        }
    }
}
