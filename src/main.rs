mod args;
mod config;
// mod dry_run;
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
// use dry_run::DryRun;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;
use crate::tale::Tale;

fn main() -> ExitCode {
    let args = Args::parse();
    let fname = PathBuf::from(args.input());

    if args.init() {
        return init::init(&fname);
    }

    let lie = match config::read(fname, args.unrestricted()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    if args.dry_run() {
        todo!();
        // println!("{}", lie.dry_run());
        // return ExitCode::SUCCESS;
    }

    let tale = Tale::from(lie);
    match tale.tell(&mut stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
