#[derive(Debug, PartialEq)]
pub(super) enum Command {
    Uci,
    Debug {
        on: bool,
    },
    IsReady,
    SetOption {
        option: EngineOption,
        value: OptionValue,
    },
    SetPosition {
        fen: Option<String>,
        moves: Vec<String>,
    },
    NewGame,
    Go {
        depth: Option<u32>,
        wtime: Option<u32>,
        btime: Option<u32>,
        winc: Option<u32>,
        binc: Option<u32>,
        nodes: Option<u32>,
        mate: Option<u32>,
        movetime: Option<u32>,
        infinite: bool,
    },
    Stop,
    Quit,
    Unknown(String),
}

#[derive(Debug, PartialEq)]
pub(super) enum EngineOption {
    Hash,
    SyzygyTablebase,
    Threads,
}

#[derive(Debug, PartialEq)]
pub(super) enum OptionValue {
    Integer(usize),
    String(String),
}

fn parse_go(parts: &[&str]) -> Command {
    let mut depth = None;
    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    let mut nodes = None;
    let mut mate = None;
    let mut movetime = None;
    let mut infinite = false;

    let mut i = 1;

    while i < parts.len() {
        match parts[i] {
            "depth" if i + 1 < parts.len() => depth = parts[i + 1].parse().ok(),
            "wtime" if i + 1 < parts.len() => wtime = parts[i + 1].parse().ok(),
            "btime" if i + 1 < parts.len() => btime = parts[i + 1].parse().ok(),
            "winc" if i + 1 < parts.len() => winc = parts[i + 1].parse().ok(),
            "binc" if i + 1 < parts.len() => binc = parts[i + 1].parse().ok(),
            "nodes" if i + 1 < parts.len() => nodes = parts[i + 1].parse().ok(),
            "mate" if i + 1 < parts.len() => mate = parts[i + 1].parse().ok(),
            "movetime" if i + 1 < parts.len() => movetime = parts[i + 1].parse().ok(),
            "infinite" => infinite = true,
            _ => {},
        }
        if parts[i] == "infinite" {
            i += 1;
        } else {
            i += 2;
        }
    }

    Command::Go {
        depth,
        wtime,
        btime,
        winc,
        binc,
        nodes,
        mate,
        movetime,
        infinite,
    }
}

fn parse_setoption(parts: &[&str]) -> Command {
    if parts.len() > 3 && parts[1] == "name" {
        let name_end = parts
            .iter()
            .position(|&x| x == "value")
            .unwrap_or(parts.len());
        let option = parts[2..name_end].join(" ");
        let option = match option.as_str() {
            "Hash" => EngineOption::Hash,
            "SyzygyTablebase" => EngineOption::SyzygyTablebase,
            "Threads" => EngineOption::Threads,
            _ => return Command::Unknown(parts.join(" ")),
        };
        let value = if name_end < parts.len() {
            match option {
                EngineOption::Hash | EngineOption::Threads => parts[name_end + 1]
                    .parse::<usize>()
                    .ok()
                    .map(OptionValue::Integer),
                EngineOption::SyzygyTablebase => {
                    Some(OptionValue::String(parts[name_end + 1..].join(" ")))
                },
            }
        } else {
            None
        };
        if let Some(value) = value {
            Command::SetOption { option, value }
        } else {
            Command::Unknown(parts.join(" "))
        }
    } else {
        Command::Unknown(parts.join(" "))
    }
}

fn parse_setposition(parts: &[&str]) -> Command {
    let fen_index = parts.iter().position(|&x| x == "fen");
    let moves_index = parts.iter().position(|&x| x == "moves");
    let fen = fen_index.map(|index| parts[index + 1..moves_index.unwrap_or(parts.len())].join(" "));
    let moves = if let Some(moves_index) = moves_index {
        parts[moves_index + 1..]
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };
    Command::SetPosition { fen, moves }
}

impl Command {
    pub(super) fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Command::Unknown(input.to_string());
        }

        match parts[0] {
            "uci" => Command::Uci,
            "debug" if parts.len() > 1 => Command::Debug {
                on: parts[1] == "on",
            },
            "isready" => Command::IsReady,
            "setoption" => parse_setoption(&parts),
            "position" => parse_setposition(&parts),
            "ucinewgame" => Command::NewGame,
            "go" => parse_go(&parts),
            "stop" => Command::Stop,
            "quit" => Command::Quit,
            _ => Command::Unknown(input.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uci() {
        assert_eq!(Command::parse("uci"), Command::Uci);
    }

    #[test]
    fn parse_debug() {
        assert_eq!(Command::parse("debug on"), Command::Debug { on: true });
        assert_eq!(Command::parse("debug off"), Command::Debug { on: false });
    }

    #[test]
    fn parse_isready() {
        assert_eq!(Command::parse("isready"), Command::IsReady);
    }

    #[test]
    fn parse_setoption() {
        assert_eq!(
            Command::parse("setoption name Hash value 128"),
            Command::SetOption {
                option: EngineOption::Hash,
                value: OptionValue::Integer(128)
            }
        );
        assert_eq!(
            Command::parse("setoption name SyzygyTablebase value /path/to/tablebase"),
            Command::SetOption {
                option: EngineOption::SyzygyTablebase,
                value: OptionValue::String("/path/to/tablebase".to_string())
            }
        );
        assert_eq!(
            Command::parse("setoption name Threads value 4"),
            Command::SetOption {
                option: EngineOption::Threads,
                value: OptionValue::Integer(4)
            }
        );
        assert_eq!(
            Command::parse("setoption name InvalidOption value 123"),
            Command::Unknown("setoption name InvalidOption value 123".to_string())
        );
    }

    #[test]
    fn parse_position() {
        assert_eq!(
            Command::parse("position startpos moves e2e4 e7e5"),
            Command::SetPosition {
                fen: None,
                moves: vec!["e2e4".to_string(), "e7e5".to_string()]
            }
        );
        assert_eq!(
            Command::parse("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4 e7e5"),
            Command::SetPosition {
                fen: Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()),
                moves: vec!["e2e4".to_string(), "e7e5".to_string()]
            }
        );
    }

    #[test]
    fn ucinewgame() {
        assert_eq!(Command::parse("ucinewgame"), Command::NewGame);
    }

    #[test]
    fn parse_go() {
        assert_eq!(Command::parse("go depth 20 wtime 300000 btime 300000 winc 10000 binc 10000 nodes 500000 mate 10 movetime 5000 infinite"),
            Command::Go {
                depth: Some(20),
                wtime: Some(300000),
                btime: Some(300000),
                winc: Some(10000),
                binc: Some(10000),
                nodes: Some(500000),
                mate: Some(10),
                movetime: Some(5000),
                infinite: true,
            });

        assert_eq!(
            Command::parse("go depth 10"),
            Command::Go {
                depth: Some(10),
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
                nodes: None,
                mate: None,
                movetime: None,
                infinite: false,
            }
        );

        assert_eq!(
            Command::parse("go wtime 1000"),
            Command::Go {
                depth: None,
                wtime: Some(1000),
                btime: None,
                winc: None,
                binc: None,
                nodes: None,
                mate: None,
                movetime: None,
                infinite: false,
            }
        );

        assert_eq!(
            Command::parse("go infinite"),
            Command::Go {
                depth: None,
                wtime: None,
                btime: None,
                winc: None,
                binc: None,
                nodes: None,
                mate: None,
                movetime: None,
                infinite: true,
            }
        );
    }

    #[test]
    fn parse_stop() {
        assert_eq!(Command::parse("stop"), Command::Stop);
    }

    #[test]
    fn parse_quit() {
        assert_eq!(Command::parse("quit"), Command::Quit);
    }

    #[test]
    fn unknown() {
        assert_eq!(
            Command::parse("unknown command"),
            Command::Unknown("unknown command".to_string())
        );
    }
}
