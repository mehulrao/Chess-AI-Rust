// This module can be used for future UCI command extensions
// Currently all commands are handled in the engine module

pub const UCI_COMMANDS: &[&str] = &[
    "uci",
    "isready",
    "ucinewgame",
    "position",
    "go",
    "stop",
    "setoption",
    "quit",
    "d",
    "display",
];

pub fn is_valid_uci_command(command: &str) -> bool {
    let first_word = command.split_whitespace().next().unwrap_or("");
    UCI_COMMANDS.contains(&first_word)
}
