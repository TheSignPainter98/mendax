mod args;
mod config;
mod dry_run;
mod error;
mod init;
mod tell;

pub use error::MendaxError;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use crate::args::Args;
use crate::tell::Tell;
use clap::Parser;
use dry_run::DryRun;
use std::io::stdout;
use std::path::PathBuf;
use std::process::ExitCode;
use tell::Style;

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
        println!("{}", lie.dry_run());
        return ExitCode::SUCCESS;
    }

    match lie.tell(&mut stdout().lock(), &mut Style::default()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
