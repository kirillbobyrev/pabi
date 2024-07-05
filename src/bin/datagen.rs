use clap::Parser;

/// Generates training data for the policy network through self-play.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Config {
    // TODO: Book to seed the starting positions from.
    // TODO: Number of games.
    // TODO: Output file.
    // TODO: Tablebase path.
    // TODO: Flatten Search config.
}

fn main() {
    let config = Config::parse();
    println!("{:?}", config);
}
