use clap::Parser;
use xz_rs::cli::*;

fn main() {
    let args = XzArgs::parse();

    if args.compress || !args.decompress {
        let _ = compress_files(&args.files).inspect_err(|e| eprintln!("{:?}", e));
    } else {
        let _ = decompress_files(&args.files).inspect_err(|e| eprintln!("{:?}", e));
    }
}
