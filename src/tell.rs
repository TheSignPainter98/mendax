use crate::config::{Colour, Fib, Lie, MendaxError, Tale};
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
    ExecutableCommand,
};
use rand::Rng;
use std::error::Error;
use std::fmt::Display;
use std::io::StdoutLock;
use std::io::Write;
use std::{thread, time::Duration};
use subprocess::Exec;

pub trait Tell {
    fn tell(&self, style: &mut Style, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>>;
}

impl Tell for Lie {
    fn tell(&self, style: &mut Style, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide, DisableBlinking)?;

        let ret = self.tale().tell(style, stdout);

        execute!(stdout, EnableBlinking, Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        ret
    }
}

impl Tell for Tale {
    fn tell(&self, style: &mut Style, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>> {
        for fib in self.fibs() {
            fib.tell(style, stdout)?;
        }

        pause()?;

        Ok(())
    }
}

impl Tell for Fib {
    fn tell(&self, style: &mut Style, stdout: &mut StdoutLock) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Run { cmd, result } => {
                if !style.screen_blank {
                    execute!(stdout, Print("\r\n"))?;
                } else {
                    style.screen_blank = false;
                }

                execute!(stdout, Print(style.ps1()))?;
                stdout.flush()?;

                execute!(stdout, Show, DisableBlinking)?;
                pause()?;

                style.fake_type(stdout, cmd.chars().collect::<Vec<_>>().as_slice())?;

                execute!(stdout, Hide, EnableBlinking)?;
                pause()?;

                for line in result {
                    execute!(stdout, Print("\r\n"), Print(line))?;
                }
                Ok(())
            }
            Self::System { apparent_cmd, cmd } => {
                if !style.screen_blank {
                    execute!(stdout, Print("\r\n"))?;
                } else {
                    style.screen_blank = false;
                }

                execute!(stdout, Print(style.ps1()))?;
                stdout.flush()?;

                pause()?;

                style.fake_type(stdout, apparent_cmd.chars().collect::<Vec<_>>().as_slice())?;

                pause()?;

                Exec::shell(cmd).join()?;

                Ok(())
            }
            Self::Look {
                speed,
                fg: _,
                bg: _,
                title,
                cwd,
                host,
                user,
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
                // TODO(kcza): colour support?

                Ok(())
            }
            Self::Clear => {
                stdout.execute(Clear(ClearType::All))?;
                style.screen_blank = true;
                Ok(())
            }
        }
    }
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
    screen_blank: bool,
    speed: f64,

    #[allow(unused)]
    fg: Colour,

    #[allow(unused)]
    bg: Colour,
    cwd: String,
    host: String,
    user: String,
}

impl Style {
    fn ps1(&self) -> String {
        format!("{}@{}:{}$ ", self.user, self.host, self.cwd)
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
            screen_blank: true,
            speed: 0.040,
            fg: Colour::White,
            bg: Colour::Black,
            cwd: "~".into(),
            host: "ubuntu".into(),
            user: "ubuntu".into(),
        }
    }
}
