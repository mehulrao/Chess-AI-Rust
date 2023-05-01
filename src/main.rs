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
    let mut board = Board::default();

    if board.status() == BoardStatus::Checkmate {
        println!("Mate!");
        return;
    }

    let mut searcher: Searcher = Searcher::new(board, true, 67108864);
    searcher.do_iterative_deepening_search(10);
    println!("{:?}", searcher.get_best_move().unwrap());
    board = board.make_move_new(searcher.get_best_move().unwrap());
    println!("--------------------------------");
    println!("{}", board);
}
