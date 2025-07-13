use chess::{CacheTable, ChessMove, Color, Game};
use std::env;

mod searcher;
use crate::{entry::Entry, searcher::Searcher};
mod entry;
mod evaluation;
mod move_ordering;
mod uci;

const TARGET_DEPTH: usize = 5;
const PLAYER: Color = Color::White;
const TT_SIZE: usize = 67108864;
const TIME: u64 = 5000;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if interactive mode is requested
    if args.len() > 1 && (args[1] == "interactive" || args[1] == "--interactive" || args[1] == "-i")
    {
        // Interactive mode
        run_interactive_mode();
    } else {
        // UCI mode (default)
        uci::run_uci_loop();
    }
}

fn run_interactive_mode() {
    let mut game = Game::new();
    //let mut game = Game::from_str("r3kb1r/pqp1n1p1/2p1b2p/4Bp2/Q3p3/P1N4N/2P2PPP/3R1RK1 w kq - 0 1").unwrap();
    if !game.result().is_none() {
        println!("Mate!");
        return;
    }

    let mut tt: CacheTable<Entry> = CacheTable::new(TT_SIZE, Entry::new_default());
    println!("--------------------------------");
    println!("Chess AI Rust - Interactive Mode");
    println!("To use UCI mode (default), run: cargo run");
    println!("--------------------------------");
    println!("{}", game.current_position());
    while game.result().is_none() {
        if PLAYER == Color::Black {
            do_search(&mut game, &mut tt);
            user_move(&mut game);
        } else {
            loop {
                user_move(&mut game);
                do_search(&mut game, &mut tt);
            }
        }
    }
}

fn user_move(game: &mut Game) {
    loop {
        let mut move_text = String::new();
        println!("Enter your move: ");
        std::io::stdin().read_line(&mut move_text).unwrap();
        if move_text.trim_end() == "O-O" {
            if PLAYER == Color::White {
                move_text = String::from("e1g1");
            } else {
                move_text = String::from("e8g8");
            }
        } else if move_text == "O-O-O" {
            if PLAYER == Color::White {
                move_text = String::from("e1c1");
            } else {
                move_text = String::from("e8c8");
            }
        }
        let _move =
            match ChessMove::from_san(&game.current_position(), &move_text.trim().to_string()) {
                Ok(m) => {
                    game.make_move(m);
                    break;
                }
                Err(_) => {
                    println!("Invalid Move: {}", move_text);
                    continue;
                }
            };
    }
    println!("--------------------------------");
    println!("{}", game.current_position());
}

fn do_search(game: &mut Game, tt: &mut CacheTable<Entry>) {
    let mut searcher: Searcher = Searcher::new(game.current_position(), true);
    searcher.do_iterative_deepening_search(TARGET_DEPTH, tt);

    println!(
        "Best Move: {}{}",
        searcher.get_best_move().unwrap().get_source(),
        searcher.get_best_move().unwrap().get_dest()
    );
    println!("Best Eval: {}", searcher.get_best_eval());
    println!("TT Hits: {}", searcher.get_num_tt());
    game.make_move(searcher.get_best_move().unwrap());
    println!("--------------------------------");
    println!("{}", game.current_position());
}
