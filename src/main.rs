mod entry;
mod evaluation;
mod move_ordering;
mod searcher;
mod uci;

fn main() {
    // Always run UCI mode
    uci::run_uci_loop();
}
