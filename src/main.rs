use chess::{Board, Color, BoardStatus, Game};
mod searcher;
use crate::searcher::Searcher;
mod entry;
mod evaluation;
mod move_ordering;

fn main() {
    const PLAYER: Color = Color::White;

    const TIME: u64 = 5000;

    //let board = Board::default();
    let mut game = Game::new();

    if !game.result().is_none() {
        println!("Mate!");
        return;
    }

    let mut searcher: Searcher = Searcher::new(game.current_position(), true, 67108864);
    searcher.do_iterative_deepening_search(7);
    println!("Best Move: {:?}", searcher.get_best_move().unwrap());
    println!("Best Eval: {}", searcher.get_best_eval());
    game.make_move(searcher.get_best_move().unwrap());
    println!("--------------------------------");
    println!("{}", game.current_position());
}
