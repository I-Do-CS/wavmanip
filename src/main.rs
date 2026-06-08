use clap::Parser;
use wavmanip::cli::Cli;

fn main() {
    if let Err(err) = wavmanip::run(Cli::parse()) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
