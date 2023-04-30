use std::str::FromStr;

use chess::{Board, Color, BoardStatus};
mod searcher;
use crate::searcher::Searcher;
mod entry;
mod evaluation;
mod move_ordering;

fn main() {
    const PLAYER: Color = Color::White;

    const TIME: u64 = 5000;

    //let board = Board::default();
    let board = Board::from_fen("rnbqkbnr/ppp2ppp/8/1B1pp3/3PP3/8/PPP2PPP/RNBQK1NR b KQkq - 0 1".to_owned()).expect("Valid FEN");

    if board.status() == BoardStatus::Checkmate {
        println!("Mate!");
        return;
    }

    let mut searcher: Searcher = Searcher::new(board, true, 67108864);
    searcher.do_iterative_deepening_search(1);
    //println!("{}", searcher.get_board());
}
