# Chess AI Rust

A chess AI engine implemented in Rust with UCI (Universal Chess Interface) support.

## Features

- **Minimax Search Algorithm** with alpha-beta pruning
- **Iterative Deepening** for time-controlled searches
- **Transposition Tables** for move caching and optimization
- **Move Ordering** for better search efficiency
- **UCI Protocol Support** - Compatible with any UCI chess interface
- **Interactive Mode** - Play directly in the terminal

## UCI Mode

The engine supports the Universal Chess Interface (UCI) protocol, making it compatible with popular chess GUIs and analysis tools.

### Usage

To run the engine in UCI mode:

```bash
cargo run --release -- uci
```

### Compatible Software

The engine works with any UCI-compatible chess software, including:

- **chess-tui** - Terminal-based chess interface
- **Arena** - Chess GUI
- **ChessBase** - Professional chess software
- **Lichess** - Online chess platform (analysis)
- **cutechess-cli** - Command-line tournament manager

### Using with chess-tui

1. Build the engine:
   ```bash
   cargo build --release
   ```

2. Run chess-tui with your engine:
   ```bash
   chess-tui -e ./target/release/chess_ai_rust
   ```

### UCI Commands Supported

- `uci` - Initialize UCI mode
- `isready` - Check if engine is ready
- `ucinewgame` - Start a new game
- `position [fen <fen> | startpos] moves <move1> <move2> ...` - Set position
- `go [depth <d>] [movetime <ms>] [wtime <ms>] [btime <ms>] [winc <ms>] [binc <ms>]` - Start searching
- `stop` - Stop current search
- `setoption name <name> value <value>` - Set engine options
- `quit` - Exit the engine

### Engine Options

- **Hash** - Transposition table size in MB (default: 64, range: 1-1024)
- **MaxDepth** - Maximum search depth (default: 10, range: 1-100)
- **UseSecondSearch** - Enable quiescence search (default: true)

## Interactive Mode

To play directly in the terminal:

```bash
cargo run --release
```

This mode provides a simple text-based interface for playing against the AI.

## Building

```bash
cargo build --release
```

## Testing

To test the UCI implementation:

```bash
# Start the engine in UCI mode
cargo run --release -- uci

# Send UCI commands manually:
# uci
# isready
# position startpos moves e2e4
# go depth 5
# quit
```

## Performance

The engine uses several optimization techniques:

- **Alpha-beta pruning** - Reduces search tree size
- **Iterative deepening** - Provides anytime results
- **Transposition tables** - Avoids re-computing positions
- **Move ordering** - Improves pruning efficiency
- **Quiescence search** - Handles tactical positions

## Architecture

- `src/main.rs` - Mode selection and interactive interface
- `src/uci/` - UCI protocol implementation
- `src/searcher.rs` - Search algorithm implementation
- `src/evaluation.rs` - Position evaluation
- `src/move_ordering.rs` - Move ordering heuristics
- `src/entry.rs` - Transposition table entries

## License

This project is licensed under the MIT License.
