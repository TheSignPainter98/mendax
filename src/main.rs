mod args;
// mod spoof;
mod config;

use crate::args::Args;
use clap::Parser;
// use crate::spoof::Spoof;
// use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

const EXAMPLE: &str = r#"
- cmd: mendax --help
- print: |
    A CLI spoofer

    Usage: mendax [OPTIONS] [file]

    Arguments:
      [file]  YAML file describing the CLI to spoof [default: cli.yml]

    Options:
          --dir <DIR>             The current working directory of the fake command-line user [default: ~]
          --host <HOST>           The host name of the fake command-line machine [env: HOST=] [default: ubuntu]
          --typing-interval <ms>  The average time between typed characters [default: 45]
          --user <USER>           The username of the fake command-line user [env: USER=kcza] [default: ubuntu]
      -h, --help                  Print help information
      -V, --version               Print version information
- cmd: ls
- print: |
    cli.yml
- cmd: cat cli.yml
- print:
    - "- cmd: mendax --help"
    - "- print: |"
    - "    A CLI spoofer"
    - "..."
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
    let f = File::create(fname)?;
    let mut w = BufWriter::new(f);

    write!(w, "{}", &EXAMPLE[1..])?;

    Ok(())
}
