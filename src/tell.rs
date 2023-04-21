use crate::{
    config::{Fib, Lie, Tale},
    MendaxError,
};
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, Hide, MoveTo, RestorePosition, SavePosition, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Attribute, Print, SetAttribute, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use rand::Rng;
use std::error::Error;
use std::fmt::Display;
use std::io::{Read, StdoutLock, Write};
use std::{thread, time::Duration};
use subprocess::{Exec, Redirection};

pub trait Tell {
    fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>>;
}

impl Tell for Lie {
    fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            Hide,
            DisableBlinking,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;

        let ret = self.tale().tell(stdout, style);

        if style.final_prompt {
            Fib::Run {
                cmd: String::new(),
                result: vec![],
            }
            .tell(stdout, style)?;
        }

        if style.insert_newline {
            execute!(stdout, Print("\r\n"))?;
        }

        execute!(stdout, EnableBlinking, Show)?;
        terminal::disable_raw_mode()?;

        ret
    }
}

impl Tell for Tale {
    fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
        for fib in self.fibs() {
            fib.tell(stdout, style)?;
        }

        Ok(())
    }
}

impl Tell for Fib {
    fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Run { cmd, result } => {
                if style.insert_newline {
                    execute!(stdout, Print("\r\n"))?;
                }

                prompt(stdout, style, cmd)?;

                for line in result {
                    execute!(stdout, Print(line), Print("\r\n"))?;
                }

                style.insert_newline = false;

                Ok(())
            }
            Self::Show { text } => {
                if style.insert_newline {
                    execute!(stdout, Print("\r\n"))?;
                }

                execute!(stdout, Print(text), Print("\r\n"))?;
                style.insert_newline = false;

                Ok(())
            }
            Self::System { apparent_cmd, cmd } => {
                if style.insert_newline {
                    execute!(stdout, Print("\r\n"))?;
                }

                prompt(stdout, style, apparent_cmd.as_ref().unwrap_or(cmd))?;

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
                style.insert_newline = false;

                Ok(())
            }
            Self::Screen { apparent_cmd, tale } => {
                if style.insert_newline {
                    execute!(stdout, Print("\r\n"))?;
                }

                if let Some(apparent_cmd) = apparent_cmd {
                    prompt(stdout, style, apparent_cmd)?;
                }

                execute!(stdout, EnterAlternateScreen, SavePosition, MoveTo(0, 0))?;

                style.insert_newline = false;
                let ret = tale.tell(stdout, style);

                pause()?;

                execute!(stdout, RestorePosition, LeaveAlternateScreen)?;
                ret
            }
            Self::Look {
                speed,
                title,
                cwd,
                host,
                user,
                final_prompt,
            } => {
                if let Some(title) = title {
                    execute!(stdout, SetTitle(title)).unwrap();
                }
                if let Some(cwd) = cwd {
                    style.cwd = cwd.clone();
                }
                if let Some(host) = host {
                    style.host = host.clone();
                }
                if let Some(user) = user {
                    style.user = user.clone();
                }
                if let Some(speed) = speed {
                    style.speed = *speed;
                }
                if let Some(final_prompt) = final_prompt {
                    style.final_prompt = *final_prompt;
                }
                // TODO(kcza): colour support?

                Ok(())
            }
            Self::Clear => {
                execute!(stdout, Clear(ClearType::All))?;
                style.insert_newline = false;
                Ok(())
            }
        }
    }
}

fn prompt(stdout: &mut StdoutLock, style: &mut Style, cmd: &str) -> Result<(), Box<dyn Error>> {
    execute!(stdout, Print(style.ps1()))?;
    execute!(stdout, Show, DisableBlinking)?;
    stdout.flush()?;

    pause()?;
    if !cmd.is_empty() {
        style.fake_type(stdout, cmd.chars().collect::<Vec<_>>().as_slice())?;
        pause()?;
    }
    execute!(stdout, Hide, EnableBlinking)?;
    execute!(stdout, Print("\r\n"))?;
    stdout.flush()?;

    Ok(())
}

fn pause() -> Result<(), Box<dyn Error>> {
    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => return Err(Box::new(MendaxError::KeyboardInterrupt)),
            Event::Key(KeyEvent { .. }) => return Ok(()),
            _ => {}
        }
    }
}

pub struct Style {
    insert_newline: bool,
    speed: f64,
    cwd: String,
    host: String,
    user: String,
    final_prompt: bool,
}

impl Style {
    fn ps1(&self) -> String {
        format!(
            "{}{}{}:{}$ ",
            self.user.as_str().bold().green(),
            "@".bold().green(),
            self.host.as_str().bold().green(),
            self.cwd.as_str().blue().bold()
        )
    }

    fn fake_type<T: Display>(
        &self,
        stdout: &mut StdoutLock,
        ts: &[T],
    ) -> Result<(), Box<dyn Error>> {
        let mut rng = rand::thread_rng();

        for t in ts.iter() {
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

impl Default for Style {
    fn default() -> Self {
        Self {
            insert_newline: false,
            speed: 0.040,
            cwd: "~".into(),
            host: "ubuntu".into(),
            user: "ubuntu".into(),
            final_prompt: true,
        }
    }
}
