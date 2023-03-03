mod args;
// mod spoof;
mod config;

use crate::args::Args;
use clap::Parser;
// use crate::spoof::Spoof;
// use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

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

fn main() {
    let args = Args::parse();
    let fname = args.input();

    if args.init().is_some() {
        return init_example();
    }

    for part in config::read(fname, args.unrestricted()).unwrap().fibs() {
        println!("{part:?}");
    }

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

fn init_example() {
    let f = File::create("cli.yml").unwrap();
    let mut w = BufWriter::new(f);
    write!(w, "{}", &EXAMPLE[1..]).unwrap();
    eprintln!("created example demo in 'cli.yml'\ncall `mendax` to run it");
}
