use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::entry::Entry;
use crate::searcher::Searcher;
use crate::uci::protocol::GoParams;
use crate::uci::utils::{build_pv_line, move_to_uci_string};
use chess::{Board, CacheTable, MoveGen};

pub struct SearchManager {
    pub is_searching: Arc<AtomicBool>,
    pub search_handle: Option<thread::JoinHandle<()>>,
    pub tt_receiver: Option<mpsc::Receiver<CacheTable<Entry>>>,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            is_searching: Arc::new(AtomicBool::new(false)),
            search_handle: None,
            tt_receiver: None,
        }
    }

    pub fn start_search(
        &mut self,
        board: Board,
        params: GoParams,
        mut tt: CacheTable<Entry>,
        max_depth: u8,
        use_second_search: bool,
        debug_mode: bool,
    ) {
        if debug_mode {
            println!(
                "info string Starting search on position: {}",
                board.to_string()
            );

            // Check if there are legal moves
            let legal_moves: Vec<chess::ChessMove> = MoveGen::new_legal(&board).collect();
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

        // Use the provided transposition table instead of creating new one
        let is_searching = self.is_searching.clone();

        // Determine search parameters
        let search_depth = params.depth.unwrap_or(max_depth) as usize;
        let time_limit = calculate_time_limit(&params, &board);

        is_searching.store(true, Ordering::Relaxed);

        let (tx, rx) = mpsc::channel();
        self.tt_receiver = Some(rx);

        let handle = thread::spawn(move || {
            // Create searcher with appropriate stop flag for infinite search
            let searcher = if params.infinite {
                Searcher::new_with_stop_flag(board, use_second_search, is_searching.clone())
            } else {
                Searcher::new(board, use_second_search)
            };
            let start_time = Instant::now();

            if params.infinite {
                execute_infinite_search(
                    searcher,
                    &mut tt,
                    start_time,
                    is_searching.clone(),
                    &params,
                );
            } else if let Some(time_limit) = time_limit {
                execute_time_based_search(
                    searcher,
                    &mut tt,
                    start_time,
                    time_limit,
                    search_depth,
                    &params,
                );
            } else {
                execute_depth_based_search(searcher, &mut tt, start_time, search_depth, &params);
            }

            io::stdout().flush().unwrap();
            is_searching.store(false, Ordering::Relaxed);
            tx.send(tt).unwrap();
        });

        self.search_handle = Some(handle);
    }

    pub fn stop_search(&mut self) {
        self.is_searching.store(false, Ordering::Relaxed);

        if let Some(handle) = self.search_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn get_tt_after_search(&mut self) -> Option<CacheTable<Entry>> {
        if let Some(receiver) = self.tt_receiver.take() {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
}

fn execute_infinite_search(
    mut searcher: Searcher,
    tt: &mut CacheTable<Entry>,
    start_time: Instant,
    is_searching: Arc<AtomicBool>,
    params: &GoParams,
) {
    // Use individual depth searches with proper UCI output
    for depth in 1..=50 {
        if !is_searching.load(Ordering::Relaxed) {
            break;
        }

        // Check node limit if specified
        if let Some(node_limit) = params.nodes {
            if searcher.get_num_nodes() as u64 >= node_limit {
                break;
            }
        }

        // Search to this exact depth (this resets searcher each time but gives us proper depth results)
        searcher.do_iterative_deepening_search(depth, tt);

        if !is_searching.load(Ordering::Relaxed) {
            break;
        }

        // Check node limit again after search
        if let Some(node_limit) = params.nodes {
            if searcher.get_num_nodes() as u64 >= node_limit {
                break;
            }
        }

        // Output UCI info for this depth
        if let Some(best_move) = searcher.get_best_move() {
            output_search_info(depth, &searcher, tt, start_time, best_move);
        }

        // Check for mate - stop if found
        if searcher.get_best_eval().abs() > (100000 - 1000) {
            break;
        }

        // Brief pause to allow stop command processing
        thread::sleep(Duration::from_millis(10));
    }

    // Send final bestmove when stopped
    output_final_bestmove(&searcher);
}

fn execute_time_based_search(
    mut searcher: Searcher,
    tt: &mut CacheTable<Entry>,
    start_time: Instant,
    time_limit: Duration,
    search_depth: usize,
    params: &GoParams,
) {
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

    // Use iterative deepening with time checks
    for depth in 1..=search_depth {
        // Check if time is up
        if search_time_up.load(Ordering::Relaxed) || start_time.elapsed() >= time_limit {
            break;
        }

        // Check node limit if specified
        if let Some(node_limit) = params.nodes {
            if searcher.get_num_nodes() as u64 >= node_limit {
                break;
            }
        }

        // Search to this depth
        searcher.do_iterative_deepening_search(depth, tt);

        // Check time again after search
        if search_time_up.load(Ordering::Relaxed) || start_time.elapsed() >= time_limit {
            break;
        }

        // Check node limit again after search
        if let Some(node_limit) = params.nodes {
            if searcher.get_num_nodes() as u64 >= node_limit {
                break;
            }
        }

        // Output UCI info for this depth
        if let Some(best_move) = searcher.get_best_move() {
            output_search_info(depth, &searcher, tt, start_time, best_move);
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
    output_final_bestmove(&searcher);
}

fn execute_depth_based_search(
    mut searcher: Searcher,
    tt: &mut CacheTable<Entry>,
    start_time: Instant,
    search_depth: usize,
    params: &GoParams,
) {
    // If mate parameter is specified, search for mate in N moves
    if let Some(mate_in_n) = params.mate {
        println!(
            "info string Starting mate search (mate in {} moves)",
            mate_in_n
        );
        io::stdout().flush().unwrap();

        // For mate search, we search progressively deeper up to mate_in_n * 2 plies
        let max_mate_depth = (mate_in_n as usize) * 2;
        for depth in 1..=max_mate_depth {
            // Check node limit if specified
            if let Some(node_limit) = params.nodes {
                if searcher.get_num_nodes() as u64 >= node_limit {
                    break;
                }
            }

            searcher.do_iterative_deepening_search(depth, tt);

            if let Some(best_move) = searcher.get_best_move() {
                output_search_info(depth, &searcher, tt, start_time, best_move);

                // Check if we found mate
                if searcher.get_best_eval().abs() > (100000 - 1000) {
                    break;
                }
            }
        }
    } else {
        println!(
            "info string Starting depth-based search (depth {})",
            search_depth
        );
        io::stdout().flush().unwrap();

        searcher.do_iterative_deepening_search(search_depth, tt);

        if let Some(best_move) = searcher.get_best_move() {
            output_search_info(search_depth, &searcher, tt, start_time, best_move);
        }
    }

    output_final_bestmove(&searcher);
}

fn output_search_info(
    depth: usize,
    searcher: &Searcher,
    tt: &CacheTable<Entry>,
    start_time: Instant,
    best_move: chess::ChessMove,
) {
    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    let nodes = searcher.get_num_nodes() as u64;
    let nps = if elapsed_ms > 0 {
        (nodes * 1000) / elapsed_ms
    } else {
        0
    };

    // Build PV line
    let pv_line = build_pv_line(&searcher.get_board(), best_move, tt, 3);

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

fn output_final_bestmove(searcher: &Searcher) {
    if let Some(best_move) = searcher.get_best_move() {
        println!("bestmove {}", move_to_uci_string(best_move));
    } else {
        println!("bestmove 0000"); // UCI spec: nullmove is "0000"
    }
}

fn calculate_time_limit(params: &GoParams, board: &Board) -> Option<Duration> {
    if let Some(movetime) = params.movetime {
        return Some(movetime);
    }

    let side_to_move = board.side_to_move();
    let (time_left, increment) = match side_to_move {
        chess::Color::White => (params.wtime, params.winc),
        chess::Color::Black => (params.btime, params.binc),
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
