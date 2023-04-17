mod args;
mod config;
mod tell;
mod dry_run;

use crate::args::Args;
use crate::tell::Tell;
use clap::Parser;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;
use tell::Style;
use dry_run::DryRun;

const EXAMPLE: &str = r#"
lie.look(#{ title: "legit demo" });

lie.run("echo Hello, world", "Hello, world");
lie.run("echo 'All of this is fake'", "'All of this is fake'");

lie.cd("~");

lie.run("ls -A1", [
    ".bash_history",
    ".bashrc",
    ".cargo",
    ".rustup",
    ".vimrc",
    ".zshrc",
    "Desktop",
    "Documents",
    "Downloads",
    "snap",
]);
"#;

fn main() -> ExitCode {
    let args = Args::parse();
    let fname = args.input();

    if args.init().is_some() {
        return match init_example(fname) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{}", e);
                ExitCode::FAILURE
            }
        };
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

fn init_example(fname: &str) -> io::Result<()> {
    let f = OpenOptions::new().create_new(true).open(fname)?;
    let mut w = BufWriter::new(f);

    write!(w, "{}", &EXAMPLE[1..])?;

    Ok(())
}
