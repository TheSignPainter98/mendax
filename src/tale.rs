use crate::error::MendaxError;
use crate::fib::Fib;
use crate::lie::Lie;
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
use subprocess::{Exec, PopenError, Redirection};

#[derive(Debug)]
pub struct Tale {
    steps: Vec<Step>,
    tags: HashMap<String, usize>,
    num_systems: usize,
}

impl From<Lie> for Tale {
    fn from(lie: Lie) -> Self {
        let mut steps = vec![];
        let mut tags = HashMap::new();
        let mut num_systems = 0;
        let mut add_final_prompt = true;
        Self::flatten(
            &mut steps,
            &mut tags,
            &mut num_systems,
            &mut add_final_prompt,
            lie.into_fibs(),
        );

        if add_final_prompt {
            steps.push(Step::Ps1);
            steps.push(Step::ShowCursor);
            steps.push(Step::Pause);
            steps.push(Step::Show("".into()));
        }

        Self {
            steps,
            tags,
            num_systems,
        }
    }
}

impl Tale {
    fn flatten(
        steps: &mut Vec<Step>,
        tags: &mut HashMap<String, usize>,
        num_systems: &mut usize,
        add_final_prompt: &mut bool,
        fibs: Vec<Fib>,
    ) {
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
                    steps.push(Step::System(System::new(cmd, *num_systems)));
                    *num_systems += 1;
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
                    Self::flatten(steps, tags, num_systems, add_final_prompt, child);
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
                        *add_final_prompt = final_prompt;
                    }
                }
                Fib::Tag { name } => {
                    tags.insert(name, steps.len());
                }
                Fib::Sleep { duration } => steps.push(Step::Sleep(duration)),
                Fib::Stop => steps.push(Step::Stop),
                Fib::Enter { msg } => {
                    steps.push(Step::Type(msg));
                    steps.push(Step::Pause);
                    steps.push(Step::Show("".into()));
                }
                Fib::Clear => steps.push(Step::Clear),
            }
        }
    }

    pub fn tell(&mut self, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>> {
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
        let mut max_system = 0;
        let mut system_cache = vec![SystemCacheEntry::default(); self.num_systems];
        while pc < self.steps.len() {
            match &self.steps[pc] {
                Step::Pause => match self.pause(stdout)? {
                    UnpauseAction::Goto(jmp) => {
                        if self.num_systems > 0 && jmp > max_system {
                            self.steps[max_system + 1..=jmp]
                                .iter()
                                .filter_map(|step| {
                                    if let Step::System(system) = step {
                                        Some(system.capture(&mut system_cache[system.id()]))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Result<_, MendaxError>>()?;

                            max_system = pc;
                        }
                        pc = jmp;
                        continue;
                    }
                    UnpauseAction::Exit => break,
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
                Step::System(system) => {
                    if max_system >= pc {
                        let cache = &system_cache[system.id()];
                        stdout.write_all(
                            cache
                                .output()
                                .expect("internal error: system command not executed"),
                        )?;
                        if cache.requires_newline() {
                            stdout.write_all(b"\r\n")?;
                        }
                        pc += 1;
                        continue;
                    }
                    max_system = pc;

                    let cache = &mut system_cache[system.id()];
                    system.stream(stdout, cache)?;
                    if cache.requires_newline() {
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
                Step::Sleep(duration) => thread::sleep(*duration),
                Step::Stop => break,
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
            }
            pc += 1;
        }

        execute!(stdout, EnableBlinking, Show)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn pause(&self, stdout: &mut StdoutLock) -> Result<UnpauseAction, Box<dyn Error>> {
        let mut printed = false;
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
                    if !printed {
                        write!(stdout, "\r\n")?;
                        printed = true;
                    }
                    write!(
                        stdout,
                        "mendax: press '/' to jump to a tag, '!' to exit, 'h' or '?' to show this help\r\n"
                        )?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('!'),
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::CONTROL,
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
                        if !printed {
                            writeln!(stdout)?;
                            printed = true;
                        }
                        if !incorrect {
                            write!(stdout, "mendax: enter tag: ")?;
                        } else {
                            write!(
                                stdout,
                                "mendax: tag incorrect; enter tag or '?' to list available: "
                            )?;
                        }
                        stdout.flush()?;

                        tag.clear();
                        io::stdin().read_line(&mut tag)?;
                        let tag = tag.trim();

                        if tag.is_empty() {
                            println!("mendax: jump cancelled");
                            return Ok(UnpauseAction::None);
                        }
                        if tag == "?" {
                            let mut known_tags =
                                self.tags.keys().map(|k| k.as_str()).collect::<Vec<_>>();
                            known_tags.sort();

                            println!("{}", known_tags.join(", "));
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
    Show(String),
    System(System),
    Sleep(Duration),
    Stop,
    Type(String),
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
}

enum UnpauseAction {
    Goto(usize),
    Exit,
    None,
}

#[derive(Debug)]
struct System {
    cmd: String,
    id: usize,
}

impl System {
    pub fn new(cmd: String, id: usize) -> Self {
        Self { cmd, id }
    }

    fn cmd(&self) -> &str {
        &self.cmd
    }

    fn id(&self) -> usize {
        self.id
    }

    pub fn capture(&self, cache: &mut SystemCacheEntry) -> Result<(), MendaxError> {
        self.exec(None, cache)
    }

    pub fn stream(
        &self,
        out: &mut StdoutLock,
        cache: &mut SystemCacheEntry,
    ) -> Result<(), MendaxError> {
        self.exec(Some(out), cache)
    }

    fn exec(
        &self,
        mut out: Option<&mut StdoutLock>,
        cache: &mut SystemCacheEntry,
    ) -> Result<(), MendaxError> {
        let mut buf = Vec::new();
        let stream = Exec::shell(self.cmd())
            .stderr(Redirection::Merge)
            .stream_stdout()?;
        let mut final_newline = false;
        let mut stdout_nonempty = false;
        for b in stream.bytes() {
            stdout_nonempty = true;
            let b = b.map_err(PopenError::from)?;
            final_newline = b == b'\n';
            if final_newline {
                if let Some(out) = &mut out {
                    out.write_all(b"\r\n").map_err(PopenError::from)?;
                }
                buf.write_all(b"\r\n").map_err(PopenError::from)?;
            } else {
                if let Some(out) = &mut out {
                    out.write_all(&[b]).map_err(PopenError::from)?;
                }
                buf.write_all(&[b]).map_err(PopenError::from)?;
            }
        }

        cache.output = Some(buf);
        cache.requires_newline = stdout_nonempty && !final_newline;

        Ok(())
    }
}

pub struct Style<'lie> {
    speed: f64,
    cwd: &'lie str,
    host: &'lie str,
    user: &'lie str,
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

#[derive(Clone, Debug, Default)]
struct SystemCacheEntry {
    output: Option<Vec<u8>>,
    requires_newline: bool,
}

impl SystemCacheEntry {
    fn output(&self) -> Option<&[u8]> {
        self.output.as_deref()
    }

    fn requires_newline(&self) -> bool {
        self.requires_newline
    }
}

impl Default for Style<'_> {
    fn default() -> Self {
        Self {
            speed: 0.040,
            cwd: "~",
            host: "ubuntu",
            user: "ubuntu",
        }
    }
}
