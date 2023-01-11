use crate::args::Args;
use ncurses::WINDOW;
use rand::Rng;
use serde::Deserialize as Deserialise;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time::Duration};

#[derive(Deserialise, Debug)]
#[serde(transparent)]
pub struct Spoof {
    #[serde(flatten)]
    stages: Vec<Stage>,
}

impl Spoof {
    pub fn from_file(src: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(src)?;
        let r = BufReader::new(file);
        Ok(serde_yaml::from_reader(r)?)
    }

    pub fn run(&self, args: &Args, window: ncurses::WINDOW) {
        for stage in &self.stages {
            stage.render(args, window)
        }
    }
}

#[derive(Deserialise, Debug)]
#[serde(untagged)]
enum Stage {
    Input(InputStage),
    Output(OutputStage),
}

impl Stage {
    fn render(&self, args: &Args, window: WINDOW) {
        match self {
            Self::Input(action) => action.render(args, window),
            Self::Output(action) => action.render(args, window),
        }
    }
}

#[derive(Deserialise, Debug)]
struct InputStage {
    cmd: String,
    prompt: Option<String>,
    dir: Option<String>,
}

impl InputStage {
    fn render(&self, args: &Args, window: WINDOW) {
        match &self.prompt {
            Some(p) => ncurses::waddstr(window, p),
            None => ncurses::waddstr(window, &args.ps1(self.dir.as_ref().map(|s| &s[..]))),
        };

        ncurses::wgetch(window);

        let interval = args.typing_interval();
        let interval_duration = Duration::from_millis(interval.into());
        for c in self.cmd.chars() {
            ncurses::waddch(window, c as u32);
            ncurses::wrefresh(window);
            thread::sleep(interval_duration);
        }

        thread::sleep(interval_duration * 3);
        move_cursor_down(window);
    }
}

#[derive(Deserialise, Debug)]
#[serde(untagged)]
enum OutputStage {
    Lines {
        #[serde(default)]
        speed: OutputSpeed,
        print: OutputLines,
    },
    Screen {
        screen: String,
    },
}

impl OutputStage {
    fn render(&self, _args: &Args, window: WINDOW) {
        match self {
            Self::Lines { print, speed, .. } => print.render(speed, window),
            Self::Screen { screen, .. } => {
                ncurses::waddstr(window, screen);
            }
        }
    }
}

#[derive(Deserialise, Debug, Eq, PartialEq)]
#[serde(untagged, rename_all = "lowercase")]
enum OutputSpeed {
    Graduated(OutputSpeedGraduations),
    Custom(u32),
}

impl OutputSpeed {
    fn mean_interval(&self) -> u32 {
        match self {
            Self::Graduated(l) => l.mean_interval(),
            Self::Custom(m) => *m,
        }
    }
}

impl Default for OutputSpeed {
    fn default() -> Self {
        Self::Graduated(OutputSpeedGraduations::default())
    }
}

#[derive(Deserialise, Debug, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum OutputSpeedGraduations {
    Fast,
    #[default]
    Medium,
    Leisurely,
    Slow,
    Tortoise,
    Snail,
}

impl OutputSpeedGraduations {
    fn mean_interval(&self) -> u32 {
        match self {
            Self::Fast => 20,
            Self::Medium => 40,
            Self::Leisurely => 100,
            Self::Slow => 250,
            Self::Tortoise => 500,
            Self::Snail => 1333,
        }
    }
}

#[derive(Deserialise, Debug)]
#[serde(untagged)]
enum OutputLines {
    Block(String),
    LineByLine(Vec<String>),
}

impl OutputLines {
    fn render(&self, speed: &OutputSpeed, window: WINDOW) {
        match self {
            Self::Block(s) => {
                if s.chars().last() == Some('\n') {
                    ncurses::waddstr(window, s);
                } else {
                    ncurses::waddstr(window, s);
                    move_cursor_down(window);
                }
            }
            Self::LineByLine(lines) => {
                let mut rng = rand::thread_rng();
                for (i, chunk) in lines.iter().enumerate() {
                    if i > 0 {
                        let mean = speed.mean_interval();
                        if mean != 0 {
                            let mean = mean as f64;
                            let deviation = mean * 0.5;
                            let interval = rng.gen_range(mean - deviation..mean + deviation) as u64;
                            thread::sleep(Duration::from_millis(interval));
                        }
                    }
                    ncurses::waddstr(window, chunk);
                    move_cursor_down(window);
                    ncurses::wrefresh(window);
                }
            }
        }
    }
}

fn move_cursor_down(window: WINDOW) {
    let mut x = 0;
    let mut y = 0;
    let mut _maxx = 0;
    let mut maxy = 0;

    ncurses::getyx(window, &mut y, &mut x);
    ncurses::getmaxyx(window, &mut maxy, &mut _maxx);

    if y == maxy - 1{
        ncurses::scroll(window);
        ncurses::wmove(window, y, 0);
    } else {
        ncurses::wmove(window, y + 1, 0);
    }
}
