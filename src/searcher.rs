use crate::{entry::Entry, evaluation::Evaluation};
use super::move_ordering::MoveOrdering;
use chess::Board;
use chess::BoardStatus;
use chess::ChessMove;
use chess::CacheTable;
use chess::MoveGen;

const IMMEDIATE_MATE_SCORE: i32 = 100000;
const POS_INF: i32 = 9999999;
const NEG_INF: i32 = -POS_INF;
const MAX_MATE_DEPTH: i32 = 1000;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum EvalType {
    Exact,
    LowerBound,
    UpperBound,
}
const INVALID_MOVE: Option<ChessMove> = None;

pub struct Searcher {
    use_second_search: bool,
    board: Board,
    tt: CacheTable<Entry>,
    best_move: Option<ChessMove>,
    num_pos: i32,
    num_nodes: i32,
    num_safe_pos: i32,
    num_prunes: i32,
    num_tt: i32,
    abort_search: bool,
    best_eval: i32,
    best_move_this_iter: Option<ChessMove>,
    best_eval_this_iter: i32,
    move_ordering: MoveOrdering,
}

impl Searcher {

    pub fn new(board: Board, use_second_search: bool, tt_size: usize) -> Searcher {
        Searcher {
            use_second_search,
            board,
            tt: CacheTable::new(tt_size, Entry::new_default()),
            best_move: None,
            num_nodes: 0,
            num_pos: 0,
            num_safe_pos: 0,
            num_prunes: 0,
            num_tt: 0,
            abort_search: false,
            best_eval: 0,
            best_move_this_iter: None,
            best_eval_this_iter: 0,
            move_ordering: MoveOrdering::new(self.tt),
        }
    }

    pub fn search_moves(&mut self, depth: u8, ply_from_root: i32, mut alpha: i32, mut beta: i32) -> i32 {
        if self.abort_search {return 0}
        if ply_from_root > 0 {
            if self.board.status() == chess::BoardStatus::Stalemate {
                return 0
            }
            alpha = alpha.max(-IMMEDIATE_MATE_SCORE + ply_from_root);
            beta = beta.min(IMMEDIATE_MATE_SCORE - ply_from_root);
            if alpha >= beta {
                return alpha
            }
        }
        let board_hash = self.board.get_hash();
        let tt_eval = self.lookup_evaluation(board_hash, depth, ply_from_root, alpha, beta);
        if tt_eval.is_some() {
            self.num_tt += 1;
            if ply_from_root == 0 {
                self.best_move_this_iter = self.get_stored_move(board_hash);
                self.best_eval_this_iter = tt_eval.unwrap();
            }
            return tt_eval.unwrap()
        }
        if depth == 0 {
            if self.use_second_search {
                return self.search_captures(alpha, beta)
            } else {
                let eval = Evaluation::new(self.board);
                return eval.evaluate(); 
            }
        }
        let mut move_list = MoveGen::new_legal(&self.board);
        let status = self.board.status();
        if status == BoardStatus::Checkmate {
            let mate_score = IMMEDIATE_MATE_SCORE - ply_from_root;
            return -mate_score;
        } else if status == BoardStatus::Stalemate {
            0;
        }

        let mut eval_type = EvalType::UpperBound;
        let mut best_move_this_pos = INVALID_MOVE;

        for this_move in move_list {
            let board_backup = self.board.clone();
            let new_board = self.board.make_move_new(this_move);
            self.board = new_board;
            let evaluation = -self.search_moves(depth - 1, ply_from_root + 1, -beta, -alpha);
            self.board = board_backup;
            self.num_nodes += 1;
            if evaluation >= beta {
                let entry: Entry = Entry::new(self.correct_score_to_store(evaluation, ply_from_root), this_move, depth, EvalType::LowerBound);
                self.tt.add(self.board.get_hash(), entry);
                return beta;
            }
            if evaluation > alpha {
                eval_type = EvalType::Exact;
                best_move_this_pos = Some(this_move);
                alpha = evaluation;
                if ply_from_root == 0 {
                    self.best_move_this_iter = Some(this_move);
                    self.best_eval_this_iter = evaluation;
                }
            }
        }
        return alpha
    }

    pub fn do_iterative_deepening_search(&mut self, mut target_depth: usize) {
        self.num_nodes = 0;
        self.num_pos = 0;
        self.num_safe_pos = 0;
        self.num_tt = 0;
        self.num_prunes = 0;
        self.best_move = None;
        self.best_move_this_iter = self.best_move;
        self.best_eval = 0;
        self.best_eval_this_iter = self.best_eval;
        let mut current_iter_search_depth: usize = 0;
        self.abort_search = false;
        if target_depth == 0 {
            target_depth = usize::MAX;
        }
        for depth in 1..=target_depth {
            self.search_moves(depth as u8, 0, NEG_INF, POS_INF);
            if self.abort_search {
                break;
            } else {
                current_iter_search_depth = depth;
                println!("Current Depth: {} Num Posititon: {}", current_iter_search_depth, self.num_nodes);
                self.best_move = self.best_move_this_iter;
                self.best_eval = self.best_eval_this_iter;
                if self.is_mate_score(self.best_eval) {
                    break;
                }
            }
        }
    }

    pub fn get_best_move(&self) -> Option<ChessMove> {
        self.best_move
    }

    pub fn end_search(&mut self) {
        self.abort_search = true
    }

    pub fn get_board(&self) -> Board {
        self.board
    }
    fn is_mate_score(&self, score: i32) -> bool {
        score.abs() > IMMEDIATE_MATE_SCORE - MAX_MATE_DEPTH
    }

    fn lookup_evaluation(&self, hash: u64, depth: u8, ply_from_root: i32, alpha: i32, beta: i32) -> Option<i32> {
        let tt_eval = self.tt.get(hash);
        if tt_eval.is_some() {
            let tt_ = tt_eval.unwrap();
            if tt_.depth >= depth {
                let corrected_score = self.correct_retrieved_mate_score(tt_.value, ply_from_root);
                if tt_.node_type == EvalType::Exact {
                    return Some(corrected_score)
                }
                if tt_.node_type == EvalType::UpperBound && corrected_score <= alpha {
                    return Some(corrected_score)
                }
                if tt_.node_type == EvalType::LowerBound && corrected_score >= beta {
                    return Some(corrected_score)
                }
            }
        }
        None
    }

    fn correct_retrieved_mate_score(&self, score: i32, num_ply_searched: i32) -> i32 {
        if self.is_mate_score(score) {
            let sign = score.signum();
            return(score * sign - num_ply_searched) * sign;
        }
        score
    }

    fn correct_score_to_store(&self, score: i32, num_ply_searched: i32) -> i32 {
        0
    }

    fn get_stored_move(&self, hash: u64) -> Option<ChessMove> {
        let tt_eval = self.tt.get(hash);
        if tt_eval.is_some() {
            let tt_ = tt_eval.unwrap();
            return Some(tt_.move_)
        }
        None
    }

    fn search_captures(&self, alpha: i32, beta: i32) -> i32 {
        let eval = Evaluation::new(self.board);
        let evaluation = eval.evaluate();
        self.num_pos += 1;
        if evaluation >= beta {
            return beta
        }
    }
    fn store_eval(&self, hash: u64, depth: u8, ply_from_root: i32, eval: i32, eval_type: EvalType, this_move: ChessMove) {
        let entry: Entry = Entry::new(self.correct_score_to_store(eval, ply_from_root), this_move, depth, EvalType::LowerBound);
        self.tt.add(self.board.get_hash(), entry);
    }
}
