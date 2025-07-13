use std::io::{self, Write};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::entry::Entry;
use crate::searcher::Searcher;
use crate::uci::protocol::{GoParams, PositionParams};
use chess::{Board, CacheTable, ChessMove, Color, Game, MoveGen, Square};

pub struct EngineOptions {
    pub hash_size: usize,
    pub max_depth: u8,
    pub use_second_search: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            hash_size: 67108864, // 64MB default
            max_depth: 10,
            use_second_search: true,
        }
    }
}

pub struct UciEngine {
    game: Game,
    tt: CacheTable<Entry>,
    options: EngineOptions,
    is_searching: Arc<AtomicBool>,
    search_handle: Option<thread::JoinHandle<()>>,
    debug_mode: bool,
}

impl UciEngine {
    pub fn new() -> Self {
        let options = EngineOptions::default();
        let tt = CacheTable::new(options.hash_size, Entry::new_default());

        Self {
            game: Game::new(),
            tt,
            options,
            is_searching: Arc::new(AtomicBool::new(false)),
            search_handle: None,
            debug_mode: false,
        }
    }

    pub fn send_options(&self) {
        println!(
            "option name Hash type spin default {} min 1 max 1024",
            self.options.hash_size / 1024 / 1024
        );
        io::stdout().flush().unwrap();

        println!(
            "option name MaxDepth type spin default {} min 1 max 100",
            self.options.max_depth
        );
        io::stdout().flush().unwrap();

        println!(
            "option name UseSecondSearch type check default {}",
            self.options.use_second_search
        );
        io::stdout().flush().unwrap();
    }

    pub fn handle_command(&mut self, command: &str) {
        let tokens: Vec<&str> = command.split_whitespace().collect();

        if tokens.is_empty() {
            return;
        }

        match tokens[0] {
            "isready" => {
                println!("readyok");
                io::stdout().flush().unwrap();
            }
            "debug" => {
                if tokens.len() > 1 {
                    match tokens[1] {
                        "on" => {
                            self.debug_mode = true;
                            if self.debug_mode {
                                println!("info string Debug mode enabled");
                                io::stdout().flush().unwrap();
                            }
                        }
                        "off" => {
                            self.debug_mode = false;
                        }
                        _ => {} // Ignore invalid debug parameters
                    }
                }
            }
            "ucinewgame" => {
                self.handle_new_game();
            }
            "position" => {
                self.handle_position(command);
            }
            "go" => {
                self.handle_go(command);
            }
            "stop" => {
                self.handle_stop();
            }
            "setoption" => {
                self.handle_setoption(command);
            }
            "d" | "display" => {
                // Debug command to display current position
                println!("{}", self.game.current_position());
                io::stdout().flush().unwrap();
            }
            _ => {
                // Unknown command, ignore as per UCI spec
                if self.debug_mode {
                    println!("info string Unknown command: {}", command);
                    io::stdout().flush().unwrap();
                }
            }
        }
    }

    fn handle_new_game(&mut self) {
        self.game = Game::new();
        self.tt = CacheTable::new(self.options.hash_size, Entry::new_default());
    }

    fn handle_position(&mut self, command: &str) {
        let params = PositionParams::from_command(command);

        // Set up the game
        if let Some(fen) = params.fen {
            match Game::from_str(&fen) {
                Ok(game) => {
                    self.game = game;
                }
                Err(_) => {
                    // Send UCI error info instead of stderr
                    println!("info string Error: Invalid FEN: {}", fen);
                    io::stdout().flush().unwrap();
                    return;
                }
            }
        } else {
            self.game = Game::new();
        }

        // Apply moves
        for move_str in params.moves {
            match ChessMove::from_san(&self.game.current_position(), &move_str) {
                Ok(chess_move) => {
                    self.game.make_move(chess_move);
                }
                Err(_) => {
                    // Try UCI format (e.g., "e2e4")
                    if let Some(chess_move) =
                        parse_uci_move(&move_str, &self.game.current_position())
                    {
                        // Validate the move is legal
                        let legal_moves: Vec<ChessMove> =
                            MoveGen::new_legal(&self.game.current_position()).collect();
                        if legal_moves.contains(&chess_move) {
                            self.game.make_move(chess_move);
                        } else {
                            println!("info string Error: Illegal move: {}", move_str);
                            io::stdout().flush().unwrap();
                            return;
                        }
                    } else {
                        println!("info string Error: Invalid move format: {}", move_str);
                        io::stdout().flush().unwrap();
                        return;
                    }
                }
            }
        }
    }

    fn handle_go(&mut self, command: &str) {
        let params = GoParams::from_command(command);

        // Stop any existing search
        self.handle_stop();

        // Start new search
        self.start_search(params);
    }

    fn handle_stop(&mut self) {
        self.is_searching.store(false, Ordering::Relaxed);

        if let Some(handle) = self.search_handle.take() {
            let _ = handle.join();
        }
    }

    fn handle_setoption(&mut self, command: &str) {
        if let Some((name, value)) = crate::uci::protocol::parse_setoption(command) {
            match name.as_str() {
                "Hash" => {
                    if let Ok(hash_mb) = value.parse::<usize>() {
                        self.options.hash_size = hash_mb * 1024 * 1024;
                        self.tt = CacheTable::new(self.options.hash_size, Entry::new_default());
                    }
                }
                "MaxDepth" => {
                    if let Ok(depth) = value.parse::<u8>() {
                        self.options.max_depth = depth;
                    }
                }
                "UseSecondSearch" => {
                    self.options.use_second_search = value.to_lowercase() == "true";
                }
                _ => {
                    // Unknown option, ignore
                }
            }
        }
    }

    fn start_search(&mut self, params: GoParams) {
        let board = self.game.current_position();
        let mut searcher = Searcher::new(board, self.options.use_second_search);
        // Create new transposition table for this search since we can't clone
        let mut tt = CacheTable::new(self.options.hash_size, Entry::new_default());
        let is_searching = self.is_searching.clone();

        // Determine search parameters
        let search_depth = params.depth.unwrap_or(self.options.max_depth) as usize;
        let time_limit = self.calculate_time_limit(&params);

        is_searching.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            let start_time = Instant::now();

            if params.infinite {
                // Infinite search - run iterative deepening until stopped
                println!("info string Starting infinite search");
                io::stdout().flush().unwrap();

                for depth in 1..=50 {
                    // Reasonable upper limit
                    if !is_searching.load(Ordering::Relaxed) {
                        break;
                    }

                    searcher.do_iterative_deepening_search(depth, &mut tt);

                    if let Some(best_move) = searcher.get_best_move() {
                        let elapsed_ms = start_time.elapsed().as_millis() as u64;
                        let nodes = searcher.get_num_nodes() as u64;
                        let nps = if elapsed_ms > 0 {
                            (nodes * 1000) / elapsed_ms
                        } else {
                            0
                        };

                        println!(
                            "info depth {} score cp {} nodes {} time {} nps {} pv {}",
                            depth,
                            searcher.get_best_eval(),
                            nodes,
                            elapsed_ms,
                            nps,
                            move_to_uci_string(best_move)
                        );
                        io::stdout().flush().unwrap();
                    }

                    // Small delay to avoid overwhelming the GUI
                    thread::sleep(Duration::from_millis(10));
                }

                // Send final bestmove when stopped
                if let Some(best_move) = searcher.get_best_move() {
                    println!("bestmove {}", move_to_uci_string(best_move));
                } else {
                    println!("bestmove 0000"); // UCI spec: nullmove is "0000"
                }
            } else {
                // Regular search with specific depth/time
                let effective_depth = if time_limit.is_some() {
                    // For time-based search, use a reasonable depth that should complete quickly
                    search_depth.min(4) // Limit to depth 4 for time-based searches
                } else {
                    search_depth
                };

                // Send initial info
                println!("info string Starting search depth {}", effective_depth);
                io::stdout().flush().unwrap();

                searcher.do_iterative_deepening_search(effective_depth, &mut tt);

                // Always send bestmove, even if search was aborted
                if let Some(best_move) = searcher.get_best_move() {
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;
                    let nodes = searcher.get_num_nodes() as u64;
                    let nps = if elapsed_ms > 0 {
                        (nodes * 1000) / elapsed_ms
                    } else {
                        0
                    };

                    println!(
                        "info depth {} score cp {} nodes {} time {} nps {} pv {}",
                        effective_depth,
                        searcher.get_best_eval(),
                        nodes,
                        elapsed_ms,
                        nps,
                        move_to_uci_string(best_move)
                    );
                    io::stdout().flush().unwrap();
                    println!("bestmove {}", move_to_uci_string(best_move));
                } else {
                    println!("bestmove 0000"); // UCI spec: nullmove is "0000"
                }
            }

            io::stdout().flush().unwrap();
            is_searching.store(false, Ordering::Relaxed);
        });

        self.search_handle = Some(handle);
    }

    fn calculate_time_limit(&self, params: &GoParams) -> Option<Duration> {
        if let Some(movetime) = params.movetime {
            return Some(movetime);
        }

        let side_to_move = self.game.current_position().side_to_move();
        let (time_left, increment) = match side_to_move {
            Color::White => (params.wtime, params.winc),
            Color::Black => (params.btime, params.binc),
        };

        if let Some(time_left) = time_left {
            let base_time = time_left.as_millis() as f64;
            let increment_time = increment.map(|i| i.as_millis() as f64).unwrap_or(0.0);
            let moves_to_go = params.movestogo.unwrap_or(30) as f64;

            // Simple time management: use time_left / moves_to_go + increment
            let allocated_time = (base_time / moves_to_go + increment_time * 0.8) as u64;
            return Some(Duration::from_millis(allocated_time));
        }

        None
    }
}

fn move_to_uci_string(chess_move: ChessMove) -> String {
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

fn parse_uci_move(move_str: &str, board: &Board) -> Option<ChessMove> {
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
