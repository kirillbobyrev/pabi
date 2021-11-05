use clap::Parser;

#[derive(Parser)]
#[clap(
    name = clap::crate_name!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
)]
struct Opts {
}

fn main() {
    let opts = Opts::parse();
}
