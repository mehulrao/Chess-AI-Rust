use std::time::Duration;

#[derive(Debug, Clone)]
pub struct GoParams {
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<u32>,
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub mate: Option<u8>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

impl Default for GoParams {
    fn default() -> Self {
        Self {
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: None,
            nodes: None,
            mate: None,
            movetime: None,
            infinite: false,
        }
    }
}

impl GoParams {
    pub fn from_command(command: &str) -> Self {
        let mut params = GoParams::default();
        let tokens: Vec<&str> = command.split_whitespace().collect();

        let mut i = 1; // Skip "go"
        while i < tokens.len() {
            match tokens[i] {
                "wtime" if i + 1 < tokens.len() => {
                    if let Ok(ms) = tokens[i + 1].parse::<u64>() {
                        params.wtime = Some(Duration::from_millis(ms));
                    }
                    i += 2;
                }
                "btime" if i + 1 < tokens.len() => {
                    if let Ok(ms) = tokens[i + 1].parse::<u64>() {
                        params.btime = Some(Duration::from_millis(ms));
                    }
                    i += 2;
                }
                "winc" if i + 1 < tokens.len() => {
                    if let Ok(ms) = tokens[i + 1].parse::<u64>() {
                        params.winc = Some(Duration::from_millis(ms));
                    }
                    i += 2;
                }
                "binc" if i + 1 < tokens.len() => {
                    if let Ok(ms) = tokens[i + 1].parse::<u64>() {
                        params.binc = Some(Duration::from_millis(ms));
                    }
                    i += 2;
                }
                "movestogo" if i + 1 < tokens.len() => {
                    if let Ok(moves) = tokens[i + 1].parse::<u32>() {
                        params.movestogo = Some(moves);
                    }
                    i += 2;
                }
                "depth" if i + 1 < tokens.len() => {
                    if let Ok(depth) = tokens[i + 1].parse::<u8>() {
                        params.depth = Some(depth);
                    }
                    i += 2;
                }
                "nodes" if i + 1 < tokens.len() => {
                    if let Ok(nodes) = tokens[i + 1].parse::<u64>() {
                        params.nodes = Some(nodes);
                    }
                    i += 2;
                }
                "mate" if i + 1 < tokens.len() => {
                    if let Ok(mate) = tokens[i + 1].parse::<u8>() {
                        params.mate = Some(mate);
                    }
                    i += 2;
                }
                "movetime" if i + 1 < tokens.len() => {
                    if let Ok(ms) = tokens[i + 1].parse::<u64>() {
                        params.movetime = Some(Duration::from_millis(ms));
                    }
                    i += 2;
                }
                "infinite" => {
                    params.infinite = true;
                    i += 1;
                }
                _ => i += 1,
            }
        }

        params
    }
}

#[derive(Debug, Clone)]
pub struct PositionParams {
    pub fen: Option<String>,
    pub moves: Vec<String>,
}

impl PositionParams {
    pub fn from_command(command: &str) -> Self {
        let mut params = PositionParams {
            fen: None,
            moves: Vec::new(),
        };

        let tokens: Vec<&str> = command.split_whitespace().collect();

        if tokens.len() < 2 {
            return params;
        }

        let mut i = 1; // Skip "position"

        if tokens[i] == "startpos" {
            i += 1;
        } else if tokens[i] == "fen" {
            i += 1;
            let mut fen_parts = Vec::new();
            while i < tokens.len() && tokens[i] != "moves" {
                fen_parts.push(tokens[i]);
                i += 1;
            }
            if !fen_parts.is_empty() {
                params.fen = Some(fen_parts.join(" "));
            }
        }

        // Process moves
        if i < tokens.len() && tokens[i] == "moves" {
            i += 1;
            while i < tokens.len() {
                params.moves.push(tokens[i].to_string());
                i += 1;
            }
        }

        params
    }
}

pub fn parse_setoption(command: &str) -> Option<(String, String)> {
    let tokens: Vec<&str> = command.split_whitespace().collect();

    if tokens.len() < 4 || tokens[1] != "name" {
        return None;
    }

    let name_start = 2;
    let mut name_end = name_start;

    // Find "value" keyword
    while name_end < tokens.len() && tokens[name_end] != "value" {
        name_end += 1;
    }

    if name_end >= tokens.len() || tokens[name_end] != "value" {
        return None;
    }

    let name = tokens[name_start..name_end].join(" ");
    let value = tokens[name_end + 1..].join(" ");

    Some((name, value))
}
