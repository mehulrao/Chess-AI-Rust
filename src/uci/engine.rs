use std::io::{self, Write};
use std::str::FromStr;

use crate::entry::Entry;
use crate::uci::protocol::{GoParams, PositionParams};
use crate::uci::search_manager::SearchManager;
use crate::uci::utils::parse_uci_move;
use chess::{CacheTable, ChessMove, Game, MoveGen};

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
    search_manager: SearchManager,
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
            search_manager: SearchManager::new(),
            debug_mode: false,
        }
    }

    pub fn send_options(&self) {
        println!(
            "option name Hash type spin default {} min 1 max 1024",
            self.options.hash_size / 1024 / 1024
        );

        println!(
            "option name MaxDepth type spin default {} min 1 max 100",
            self.options.max_depth
        );

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
        // Clear TT for new game (positions from previous game are not relevant)
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

        // Stop any existing search and retrieve TT if available
        self.handle_stop();
        if let Some(updated_tt) = self.search_manager.get_tt_after_search() {
            self.tt = updated_tt;
        }

        // Move our TT to the search manager (temporarily)
        let tt_for_search = std::mem::replace(
            &mut self.tt,
            CacheTable::new(self.options.hash_size, Entry::new_default()),
        );

        // Start new search with our TT
        self.search_manager.start_search(
            self.game.current_position(),
            params,
            tt_for_search,
            self.options.max_depth,
            self.options.use_second_search,
            self.debug_mode,
        );
    }

    fn handle_stop(&mut self) {
        self.search_manager.stop_search();

        // Retrieve TT back from search manager if available
        if let Some(updated_tt) = self.search_manager.get_tt_after_search() {
            self.tt = updated_tt;
        }
    }

    fn handle_setoption(&mut self, command: &str) {
        if let Some((name, value)) = crate::uci::protocol::parse_setoption(command) {
            match name.as_str() {
                "Hash" => {
                    if let Ok(hash_mb) = value.parse::<usize>() {
                        self.options.hash_size = hash_mb * 1024 * 1024;
                        // Recreate TT with new size (this clears previous entries, which is correct)
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
}
