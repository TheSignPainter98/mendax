mod args;
mod lie;
mod dry_run;
mod error;
mod fib;
mod init;
mod tale;

pub use error::MendaxError;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use crate::args::Args;
use clap::Parser;
use dry_run::DryRun;
use crate::tale::Tale;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    let fname = PathBuf::from(args.input());

    if args.init() {
        return init::init(&fname);
    }

    let lie = match lie::read(fname, args.unrestricted()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    if args.dry_run() {
        println!("{}", lie.dry_run());
        return ExitCode::SUCCESS;
    }

    match Tale::from(lie).tell(&mut stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
