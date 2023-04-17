mod args;
mod config;
mod tell;
mod init;
mod dry_run;

use crate::args::Args;
use crate::tell::Tell;
use clap::Parser;
use std::io::stdout;
use std::process::ExitCode;
use tell::Style;
use dry_run::DryRun;

fn main() -> ExitCode {
    let args = Args::parse();
    let fname = args.input();

    if args.init() {
        return init::init(fname);
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
