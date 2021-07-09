use clap::Clap;

#[derive(Clap)]
#[clap(
    name = clap::crate_name!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
)]
struct Opts {
    /// Logging level scales from is the least verbose ("error") to the most verbose ("trace").
    #[clap(long, possible_values(&["error", "warn", "info", "debug", "trace"]),
           default_value("error"))]
    log_level: log::Level,
}

fn main() {
    let opts = Opts::parse();
    log::info!("Log level: {}", &opts.log_level);
}
