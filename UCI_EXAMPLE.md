# UCI Implementation Example

This chess engine now supports the Universal Chess Interface (UCI) protocol.

## Quick Start

1. **Build the engine:**
   ```bash
   cargo build --release
   ```

2. **Run in UCI mode:**
   ```bash
   cargo run --release -- uci
   ```

3. **Use with chess-tui:**
   ```bash
   chess-tui -e ./target/release/chess_ai_rust
   ```

## Manual UCI Testing

You can test the UCI implementation manually:

```bash
# Start the engine
cargo run --release -- uci

# The engine will respond with:
# id name Chess AI Rust
# id author Chess AI Rust Developer
# option name Hash type spin default 64 min 1 max 1024
# option name MaxDepth type spin default 10 min 1 max 100
# option name UseSecondSearch type check default true
# uciok

# Now you can send commands:
isready
# readyok

position startpos moves e2e4
go depth 5
# info depth 1 score cp 23 nodes 45 time 1 pv d7d6
# info depth 2 score cp 18 nodes 127 time 3 pv d7d6
# ...
# bestmove d7d6

quit
```

## UCI Commands Supported

- `uci` - Initialize UCI mode
- `isready` - Check if engine is ready
- `ucinewgame` - Start a new game
- `position [fen <fen> | startpos] moves <moves>` - Set position
- `go [depth <d>] [movetime <ms>] [time controls...]` - Start searching
- `stop` - Stop current search
- `setoption name <name> value <value>` - Set engine options
- `quit` - Exit

## Engine Options

- **Hash**: Transposition table size in MB (1-1024, default: 64)
- **MaxDepth**: Maximum search depth (1-100, default: 10)
- **UseSecondSearch**: Enable quiescence search (true/false, default: true)

## Interactive Mode

The original interactive mode is still available:

```bash
cargo run --release
# (runs without UCI, direct terminal play)
```
