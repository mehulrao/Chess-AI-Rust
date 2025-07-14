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
                    if self.debug_mode {
                        println!("info string Position set from FEN: {}", fen);
                        io::stdout().flush().unwrap();
                    }
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
            if self.debug_mode {
                println!("info string Position set to startpos");
                io::stdout().flush().unwrap();
            }
        }

        // Apply moves
        for move_str in params.moves {
            if self.debug_mode {
                println!("info string Applying move: {}", move_str);
                io::stdout().flush().unwrap();
            }

            match ChessMove::from_san(&self.game.current_position(), &move_str) {
                Ok(chess_move) => {
                    self.game.make_move(chess_move);
                    if self.debug_mode {
                        println!("info string Move applied successfully (SAN): {}", move_str);
                        io::stdout().flush().unwrap();
                    }
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
                            if self.debug_mode {
                                println!(
                                    "info string Move applied successfully (UCI): {}",
                                    move_str
                                );
                                io::stdout().flush().unwrap();
                            }
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

        if self.debug_mode {
            println!(
                "info string Final position: {}",
                self.game.current_position().to_string()
            );
            io::stdout().flush().unwrap();
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

        if self.debug_mode {
            println!(
                "info string Starting search on position: {}",
                board.to_string()
            );

            // Check if there are legal moves
            let legal_moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();
            println!("info string Legal moves count: {}", legal_moves.len());
            if legal_moves.len() > 0 {
                println!(
                    "info string First few legal moves: {:?}",
                    &legal_moves[..legal_moves.len().min(3)]
                );
            } else {
                println!("info string No legal moves available!");
            }
            io::stdout().flush().unwrap();
        }

        // Create new transposition table for this search since we can't clone
        let mut tt = CacheTable::new(self.options.hash_size, Entry::new_default());
        let is_searching = self.is_searching.clone();

        // Determine search parameters
        let search_depth = params.depth.unwrap_or(self.options.max_depth) as usize;
        let time_limit = self.calculate_time_limit(&params);
        let use_second_search = self.options.use_second_search;

        is_searching.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            // Create searcher with appropriate stop flag for infinite search
            let mut searcher = if params.infinite {
                Searcher::new_with_stop_flag(board, use_second_search, is_searching.clone())
            } else {
                Searcher::new(board, use_second_search)
            };
            let start_time = Instant::now();

            if params.infinite {
                // Infinite search - run iterative deepening with UCI output for each depth

                // Use individual depth searches with proper UCI output
                for depth in 1..=50 {
                    if !is_searching.load(Ordering::Relaxed) {
                        break;
                    }

                    // Search to this exact depth (this resets searcher each time but gives us proper depth results)
                    searcher.do_iterative_deepening_search(depth, &mut tt);

                    if !is_searching.load(Ordering::Relaxed) {
                        break;
                    }

                    // Output UCI info for this depth
                    if let Some(best_move) = searcher.get_best_move() {
                        let elapsed_ms = start_time.elapsed().as_millis() as u64;
                        let nodes = searcher.get_num_nodes() as u64;
                        let nps = if elapsed_ms > 0 {
                            (nodes * 1000) / elapsed_ms
                        } else {
                            0
                        };

                        // Build a simple PV line by looking ahead a few moves
                        let pv_line = build_pv_line(&searcher.get_board(), best_move, &tt, 3);

                        println!(
                            "info depth {} seldepth {} time {} nodes {} score cp {} nps {} tbhits 0 pv {}",
                            depth,
                            depth, // seldepth = depth for simplicity
                            elapsed_ms,
                            nodes,
                            searcher.get_best_eval(),
                            nps,
                            pv_line
                        );
                        io::stdout().flush().unwrap();
                    }

                    // Check for mate - stop if found
                    if searcher.get_best_eval().abs() > (100000 - 1000) {
                        break;
                    }

                    // Brief pause to allow stop command processing
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
                if let Some(time_limit) = time_limit {
                    // Time-based search - use iterative deepening with time control
                    println!(
                        "info string Starting time-based search ({}ms allocated)",
                        time_limit.as_millis()
                    );
                    io::stdout().flush().unwrap();

                    // Create a timer thread to stop search when time runs out
                    let search_time_up = Arc::new(AtomicBool::new(false));
                    let search_time_up_clone = search_time_up.clone();

                    let timer_handle = thread::spawn(move || {
                        thread::sleep(time_limit);
                        search_time_up_clone.store(true, Ordering::Relaxed);
                    });

                    // Use iterative deepening with time checks (similar to infinite search)
                    for depth in 1..=search_depth {
                        // Check if time is up
                        if search_time_up.load(Ordering::Relaxed)
                            || start_time.elapsed() >= time_limit
                        {
                            break;
                        }

                        // Search to this depth
                        searcher.do_iterative_deepening_search(depth, &mut tt);

                        // Check time again after search
                        if search_time_up.load(Ordering::Relaxed)
                            || start_time.elapsed() >= time_limit
                        {
                            break;
                        }

                        // Output UCI info for this depth
                        if let Some(best_move) = searcher.get_best_move() {
                            let elapsed_ms = start_time.elapsed().as_millis() as u64;
                            let nodes = searcher.get_num_nodes() as u64;
                            let nps = if elapsed_ms > 0 {
                                (nodes * 1000) / elapsed_ms
                            } else {
                                0
                            };

                            // Build PV line
                            let pv_line = build_pv_line(&searcher.get_board(), best_move, &tt, 3);

                            println!(
                                "info depth {} seldepth {} time {} nodes {} score cp {} nps {} tbhits 0 pv {}",
                                depth,
                                depth,
                                elapsed_ms,
                                nodes,
                                searcher.get_best_eval(),
                                nps,
                                pv_line
                            );
                            io::stdout().flush().unwrap();
                        }

                        // Check for mate - stop if found
                        if searcher.get_best_eval().abs() > (100000 - 1000) {
                            break;
                        }

                        // Time management: if we used more than 80% of time, stop
                        if start_time.elapsed() > time_limit * 4 / 5 {
                            break;
                        }
                    }

                    // Clean up timer thread
                    search_time_up.store(true, Ordering::Relaxed);
                    let _ = timer_handle.join();

                    // Send final bestmove
                    if let Some(best_move) = searcher.get_best_move() {
                        println!("bestmove {}", move_to_uci_string(best_move));
                    } else {
                        println!("bestmove 0000");
                    }
                } else {
                    // Depth-based search
                    println!(
                        "info string Starting depth-based search (depth {})",
                        search_depth
                    );
                    io::stdout().flush().unwrap();

                    searcher.do_iterative_deepening_search(search_depth, &mut tt);

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
                            search_depth,
                            searcher.get_best_eval(),
                            nodes,
                            elapsed_ms,
                            nps,
                            move_to_uci_string(best_move)
                        );
                        io::stdout().flush().unwrap();
                    }
                }

                // Always send bestmove, even if search was aborted
                if let Some(best_move) = searcher.get_best_move() {
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

fn build_pv_line(
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
