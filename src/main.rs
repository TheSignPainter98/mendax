mod args;
mod spoof;

use args::Args;
use clap::Parser;
use spoof::Spoof;

fn main() {
    let args = Args::parse();
    let fname = args.input();

    let spoof = match Spoof::from_file(fname) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to open file {}: {}", fname, e);
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
