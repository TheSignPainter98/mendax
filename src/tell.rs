use crate::config::{Colour, Fib, Lie, Tale};
use crossterm::{
    event, execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
    ExecutableCommand,
};
use lazy_static::lazy_static;
use rand::Rng;
use std::error::Error;
use std::io::StdoutLock;
use std::io::Write;
use std::{env, fmt::Display};
use std::{thread, time::Duration};
use subprocess::Exec;

pub trait Tell {
    fn tell<'a>(&self, style: &mut Style, stdout: &'a mut StdoutLock)
        -> Result<(), Box<dyn Error>>;
}

impl Tell for Lie {
    fn tell<'a>(
        &self,
        style: &mut Style,
        stdout: &'a mut StdoutLock,
    ) -> Result<(), Box<dyn Error>> {
        execute!(stdout, EnterAlternateScreen).unwrap();

        let ret = self.tale().tell(style, stdout);

        execute!(stdout, LeaveAlternateScreen).unwrap();

        ret
    }
}

impl Tell for Tale {
    fn tell<'a>(
        &self,
        style: &mut Style,
        stdout: &'a mut StdoutLock,
    ) -> Result<(), Box<dyn Error>> {
        for fib in self.fibs() {
            fib.tell(style, stdout)?;
        }

        event::read().unwrap();

        Ok(())
    }
}

impl Tell for Fib {
    fn tell<'a>(
        &self,
        style: &mut Style,
        stdout: &'a mut StdoutLock,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Run { cmd, result } => {
                write!(stdout, "{}", style.ps1())?;
                stdout.flush()?;

                event::read()?;

                style.fake_type(stdout, cmd.chars().collect::<Vec<_>>().as_slice())?;

                event::read()?;

                for line in result {
                    writeln!(stdout, "{line}")?;
                }

                Ok(())
            }
            Self::System { apparent_cmd, cmd } => {
                write!(stdout, "{}", style.ps1())?;
                stdout.flush()?;

                event::read()?;

                style.fake_type(stdout, cmd.chars().collect::<Vec<_>>().as_slice())?;

                event::read()?;

                Exec::shell(cmd).join()?;

                Ok(())
            }
            Self::Look {
                speed,
                fg,
                bg,
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
                Ok(())
            }
        }
    }
}

pub struct Style {
    speed: f64,
    fg: Colour,
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
                let interval = rng.gen_range(self.speed - deviation..self.speed + deviation) as f64;

                thread::sleep(Duration::from_millis((interval * 1000.0) as u64));
            }

            write!(stdout, "{t}")?;
            stdout.flush()?;
        }

        Ok(())
    }
}

lazy_static! {
    static ref CWD: String = env::var("CWD").unwrap_or("~".into());
    static ref HOST: String = env::var("HOST").unwrap_or("ubuntu".into());
    static ref USER: String = env::var("USER").unwrap_or("ubuntu".into());
}

impl Default for Style {
    fn default() -> Self {
        Self {
            speed: 0.040,
            fg: Colour::White,
            bg: Colour::Black,
            cwd: CWD.clone(),
            host: HOST.clone(),
            user: USER.clone(),
        }
    }
}
