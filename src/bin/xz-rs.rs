use clap::Parser;
use xz_rs::cli::{do_action, Action, Options, XzArgs};

fn main() {
    let args = XzArgs::parse();

    let options = Options {
        keep: args.keep || args.stdout,
        force: args.force,
        stdout: args.stdout,
    };

    let action = if args.list {
        Action::List
    } else if args.test {
        Action::Test
    } else if args.decompress == Some(true) && args.compress != Some(true) {
        Action::Decompress
    } else {
        Action::Compress
    };

    if let Err(e) = do_action(&action, &options, &args.files) {
        eprintln!("Error: {:?}", e);
    }
}
