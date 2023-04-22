use crate::error::MendaxError;
use crate::config::Lie;
use crate::fib::Fib;
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, Hide, MoveTo, RestorePosition, SavePosition, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Attribute, Print, SetAttribute, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, Read, StdoutLock, Write};
use std::{thread, time::Duration};
use subprocess::{Exec, Redirection};

#[derive(Debug)]
pub struct Tale {
    steps: Vec<Step>,
    tags: HashMap<String, usize>,
}

impl From<Lie> for Tale {
    fn from(lie: Lie) -> Self {
        let mut steps = vec![];
        let mut tags = HashMap::new();
        Self::flatten(&mut steps, &mut tags, lie.into_fibs());

        Self { steps, tags }
    }
}

impl Tale {
    fn flatten(steps: &mut Vec<Step>, tags: &mut HashMap<String, usize>, fibs: Vec<Fib>) {
        for fib in fibs {
            match fib {
                Fib::Run { cmd, result } => {
                    steps.push(Step::Ps1);
                    steps.push(Step::ShowCursor);
                    steps.push(Step::Pause);
                    steps.push(Step::Type(cmd));
                    steps.push(Step::Pause);
                    steps.push(Step::Show("".into()));
                    steps.push(Step::HideCursor);
                    for line in result {
                        steps.push(Step::Show(line));
                    }
                }
                Fib::Show { text } => steps.push(Step::Show(text)),
                Fib::System { apparent_cmd, cmd } => {
                    steps.push(Step::ShowCursor);
                    steps.push(Step::Ps1);
                    steps.push(Step::Pause);
                    steps.push(Step::Type(apparent_cmd.unwrap_or_else(|| cmd.clone())));
                    steps.push(Step::Pause);
                    steps.push(Step::Show("".into()));
                    steps.push(Step::HideCursor);
                    steps.push(Step::System(cmd));
                }
                Fib::Screen {
                    apparent_cmd,
                    fibs: child,
                } => {
                    if let Some(apparent_cmd) = apparent_cmd {
                        steps.push(Step::Ps1);
                        steps.push(Step::ShowCursor);
                        steps.push(Step::Pause);
                        steps.push(Step::Type(apparent_cmd));
                        steps.push(Step::Pause);
                        steps.push(Step::Show("".into()));
                        steps.push(Step::HideCursor);
                    }
                    steps.push(Step::ScreenOpen);
                    Self::flatten(steps, tags, child);
                    steps.push(Step::ShowCursor);
                    steps.push(Step::Pause);
                    steps.push(Step::ScreenClose);
                }
                Fib::Look {
                    speed,
                    title,
                    cwd,
                    host,
                    user,
                    final_prompt,
                } => {
                    if let Some(title) = title {
                        steps.push(Step::SetTitle(title));
                    }
                    if let Some(cwd) = cwd {
                        steps.push(Step::SetCwd(cwd));
                    }
                    if let Some(host) = host {
                        steps.push(Step::SetHost(host));
                    }
                    if let Some(user) = user {
                        steps.push(Step::SetUser(user));
                    }
                    if let Some(speed) = speed {
                        steps.push(Step::SetSpeed(speed));
                    }
                    if let Some(final_prompt) = final_prompt {
                        steps.push(Step::SetFinalPrompt(final_prompt));
                    }
                }
                Fib::Tag { name } => {
                    tags.insert(name, steps.len());
                }
                Fib::Clear => steps.push(Step::Clear),
            }
        }
    }

    pub fn tell(&self, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>> {
        let mut style = Style::default();

        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            Hide,
            DisableBlinking,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;

        let mut pc = 0;
        while pc < self.steps.len() {
            match &self.steps[pc] {
                Step::Pause => match self.pause(stdout)? {
                    UnpauseAction::Goto(jmp) => {
                        pc = jmp;
                        continue;
                    }
                    UnpauseAction::Exit => {
                        style.final_prompt = false;
                        break;
                    },
                    UnpauseAction::None => {}
                },
                Step::ShowCursor => {
                    execute!(stdout, Show, DisableBlinking)?;
                    stdout.flush()?;
                }
                Step::HideCursor => {
                    execute!(stdout, Hide, EnableBlinking)?;
                    stdout.flush()?;
                }
                Step::Ps1 => {
                    execute!(stdout, Print(style.ps1()))?;
                    stdout.flush()?;
                }
                Step::Type(msg) => style.fake_type(stdout, msg.chars())?,
                Step::Show(line) => {
                    execute!(stdout, Print(line), Print("\r\n"))?;
                    stdout.flush()?;
                }
                Step::System(cmd) => {
                    let out = Exec::shell(cmd)
                        .stderr(Redirection::Merge)
                        .stream_stdout()?;
                    let mut final_newline = false;
                    let mut stdout_nonempty = false;
                    for b in out.bytes() {
                        stdout_nonempty = true;
                        let b = b?;
                        if b == b'\n' {
                            final_newline = true;
                            execute!(stdout, Print("\r\n"))?;
                        } else {
                            final_newline = false;
                            stdout.write_all(&[b])?;
                        }
                    }
                    if stdout_nonempty && !final_newline {
                        execute!(
                            stdout,
                            SetAttribute(Attribute::Reverse),
                            Print("%"),
                            SetAttribute(Attribute::Reset),
                            Print("\r\n")
                        )?;
                    }
                    execute!(stdout, SetAttribute(Attribute::Reset))?;
                }
                Step::Clear => execute!(stdout, Clear(ClearType::All))?,
                Step::ScreenOpen => {
                    execute!(stdout, SavePosition, EnterAlternateScreen, MoveTo(0, 0))?
                }
                Step::ScreenClose => execute!(stdout, LeaveAlternateScreen, RestorePosition)?,

                Step::SetSpeed(speed) => style.speed = *speed,
                Step::SetTitle(title) => execute!(stdout, SetTitle(title))?,
                Step::SetCwd(cwd) => style.cwd = &cwd[..],
                Step::SetHost(host) => style.host = &host[..],
                Step::SetUser(user) => style.user = &user[..],
                Step::SetFinalPrompt(final_prompt) => style.final_prompt = *final_prompt,
            }
            pc += 1;
        }
        if style.final_prompt {
            execute!(stdout, Print(style.ps1()))?;
            self.pause(stdout)?;
        }

        if style.final_newline {
            execute!(stdout, Print("\r\n"))?;
        }

        execute!(stdout, EnableBlinking, Show)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn pause(&self, stdout: &mut StdoutLock) -> Result<UnpauseAction, Box<dyn Error>> {
        loop {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => return Err(Box::new(MendaxError::KeyboardInterrupt)),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h') | KeyCode::Char('?'),
                    ..
                }) => {
                    write!(stdout, "\r\n(help) press '/' to jump to tag, '!' to exit, 'h' for this help")?;
                    stdout.flush()?;
                },
                Event::Key(KeyEvent {
                    code: KeyCode::Char('!'),
                    ..
                }) => return Ok(UnpauseAction::Exit),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('/'),
                    ..
                }) => {
                    terminal::disable_raw_mode()?;
                    let mut incorrect = false;
                    let mut tag = String::new();
                    let pc = loop {
                        if !incorrect {
                            write!(stdout, "\nenter tag: ")?;
                        } else {
                            write!(stdout, "tag incorrect; enter tag: ")?;
                        }
                        stdout.flush()?;

                        tag.clear();
                        io::stdin().read_line(&mut tag)?;
                        let tag = tag.trim();

                        if tag == "?" {
                            let mut known_tags = self.tags.keys().map(|k| k.as_str()).collect::<Vec<_>>();
                            known_tags.sort();

                            print!("available tags: {}", known_tags.join(", "));
                            continue;
                        }
                        if let Some(pc) = self.tags.get(tag) {
                            break *pc;
                        }
                        incorrect = true;
                    };
                    terminal::enable_raw_mode()?;
                    return Ok(UnpauseAction::Goto(pc));
                }
                Event::Key(KeyEvent { .. }) => return Ok(UnpauseAction::None),
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
enum Step {
    Pause,
    Ps1,
    Type(String),
    Show(String),
    System(String),
    Clear,
    ScreenOpen,
    ScreenClose,

    ShowCursor,
    HideCursor,

    SetSpeed(f64),
    SetTitle(String),
    SetCwd(String),
    SetHost(String),
    SetUser(String),
    SetFinalPrompt(bool),
}

enum UnpauseAction {
    Goto(usize),
    Exit,
    None,
}

pub struct Style<'lie> {
    speed: f64,
    cwd: &'lie str,
    host: &'lie str,
    user: &'lie str,
    final_prompt: bool,
    final_newline: bool,
}

impl<'lie> Style<'lie> {
    fn ps1(&self) -> String {
        format!(
            "{}{}{}:{}$ ",
            self.user.bold().green(),
            "@".bold().green(),
            self.host.bold().green(),
            self.cwd.blue().bold()
        )
    }

    fn fake_type<T: Display>(
        &self,
        stdout: &mut StdoutLock,
        ts: impl Iterator<Item = T>,
    ) -> Result<(), Box<dyn Error>> {
        let mut rng = rand::thread_rng();

        for t in ts {
            if self.speed != 0.0 {
                let deviation = self.speed * 0.5;
                let interval = rng.gen_range(self.speed - deviation..self.speed + deviation);

                thread::sleep(Duration::from_millis((interval * 1000.0) as u64));
            }

            execute!(stdout, Print(t))?;
            stdout.flush()?;
        }

        Ok(())
    }
}

impl Default for Style<'_> {
    fn default() -> Self {
        Self {
            speed: 0.040,
            cwd: "~",
            host: "ubuntu",
            user: "ubuntu",
            final_prompt: true,
            final_newline: true,
        }
    }
}
