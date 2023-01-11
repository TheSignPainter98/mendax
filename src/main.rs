mod args;
mod spoof;

use args::Args;
use clap::Parser;
use spoof::Spoof;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

const EXAMPLE: &str = r"
- cmd: gcc --version
- print:
  - gcc (Ubuntu 11.3.0-1ubuntu1~22.04) 11.3.0
  - Copyright (C) 2021 Free Software Foundation, Inc.
  - This is free software; see the source for copying conditions.  There is NO
  - warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
";

fn main() {
    let args = Args::parse();
    let fname = args.input();

    if fname == "init" {
        init_example();
        return;
    }

    let spoof = match Spoof::from_file(fname) {
        Ok(s) => s,
        Err(e) => {
            if fname == "cli.yml" {
                eprintln!(
                    "no such file '{}'\nrun `{} init` to create an example file",
                    fname,
                    env::args().next().unwrap()
                );
            } else {
                eprintln!("failed to open file '{}': {}", fname, e);
            }
            return;
        }
    };

    ncurses::initscr();
    ncurses::noecho();

    let window = {
        let mut lines = 0;
        let mut cols = 0;
        ncurses::getmaxyx(ncurses::stdscr(), &mut cols, &mut lines);
        ncurses::newwin(cols, lines, 0, 0)
    };
    ncurses::scrollok(window, true);

    spoof.run(&args, window);

    // Hang before exit
    ncurses::wgetch(window);

    ncurses::delwin(window);
    ncurses::endwin();
}

fn init_example() {
    let f = File::create("cli.yml").unwrap();
    let mut w = BufWriter::new(f);
    write!(w, "{}", &EXAMPLE[1..]).unwrap();
}
