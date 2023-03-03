mod args;
// mod spoof;
mod config;

use crate::args::Args;
use clap::Parser;
// use crate::spoof::Spoof;
// use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

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

    let tale = match config::read(fname, args.unrestricted()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::FAILURE;
        }
    };

    for fib in tale.tale().fibs() {
        println!("{fib:?}");
    }

    ExitCode::SUCCESS

    // let spoof = match Spoof::from_file(fname) {
    //     Ok(s) => s,
    //     Err(e) => {
    //         if e.downcast_ref::<std::io::Error>().is_some() {
    //             eprintln!(
    //                 "no such file '{}'\nrun `{} init` to create an example file",
    //                 fname,
    //                 env::args().next().unwrap()
    //             );
    //         } else {
    //             eprintln!("error parsing file '{}': {}", fname, e);
    //         }
    //         return;
    //     }
    // };

    // ncurses::initscr();
    // ncurses::noecho();

    // let window = {
    //     let mut lines = 0;
    //     let mut cols = 0;
    //     ncurses::getmaxyx(ncurses::stdscr(), &mut cols, &mut lines);
    //     ncurses::newwin(cols, lines, 0, 0)
    // };
    // ncurses::scrollok(window, true);

    // spoof.run(&args, window);

    // // Hang before exit
    // ncurses::wgetch(window);

    // ncurses::delwin(window);
    // ncurses::endwin();
}

fn init_example(fname: &str) -> io::Result<()> {
    let f = OpenOptions::new().create_new(true).open(fname)?;
    let mut w = BufWriter::new(f);

    write!(w, "{}", &EXAMPLE[1..])?;

    Ok(())
}
