mod cli;
mod color;
mod config;
mod init;
mod layout;
mod modules;
mod prompt;
mod segment;
mod width;

fn main() {
    if let Err(error) = cli::run(std::env::args().skip(1)) {
        eprintln!("clavis-fish-theme: {error}");
        std::process::exit(2);
    }
}
