use std::path::PathBuf;

use xz_rs::cli::decompress_files;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let _ = decompress_files(&args[1..].iter().map(PathBuf::from).collect::<Vec<_>>())
        .map_err(|e| eprintln!("Error: {e}"));
}
