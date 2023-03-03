use crate::config::{Colour, Fib, Lie, Tale};
use crossterm::{
    event, execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
    ExecutableCommand,
};
use lazy_static::lazy_static;
use std::env;
use std::io::StdoutLock;
use std::io::Write;
use subprocess::Exec;

pub trait Tell {
    fn tell<'a>(&self, style: &mut Style, stdout: &'a mut StdoutLock) -> bool;
}

impl Tell for Lie {
    fn tell<'a>(&self, style: &mut Style, stdout: &'a mut StdoutLock) -> bool {
        execute!(stdout, EnterAlternateScreen).unwrap();

        let ret = self.tale().tell(style, stdout);

        execute!(stdout, LeaveAlternateScreen).unwrap();

        ret
    }
}

impl Tell for Tale {
    fn tell<'a>(&self, style: &mut Style, stdout: &'a mut StdoutLock) -> bool {
        for fib in self.fibs() {
            if !fib.tell(style, stdout) {
                return false;
            }
        }

        event::read().unwrap();

        true
    }
}

impl Tell for Fib {
    fn tell<'a>(&self, style: &mut Style, stdout: &'a mut StdoutLock) -> bool {
        match self {
            Self::Run { cmd, result } => {
                write!(stdout, "{}", style.ps1()).unwrap();
                write!(stdout, "{cmd}").unwrap();
                stdout.flush().unwrap();
                event::read().unwrap();

                for line in result {
                    writeln!(stdout, "{line}").unwrap();
                }
                true
            }
            Self::System { apparent_cmd, cmd } => {
                write!(stdout, "{}", style.ps1()).unwrap();
                write!(stdout, "{apparent_cmd}").unwrap();
                stdout.flush().unwrap();
                event::read().unwrap();

                match Exec::shell(cmd).join() {
                    Ok(_) => true,
                    Err(e) => {
                        eprintln!("{e}");
                        false
                    }
                }
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
                // TODO(kcza): colour support?

                true
            }
            // Self::Screen { tale } => tale.tell(&mut style.child(), stdout),
            Self::Clear => stdout.execute(Clear(ClearType::All)).is_ok(), // TODO(kcza): proper error handling
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
    fn child(&self) -> Self {
        Self {
            cwd: self.cwd.clone(),
            host: self.host.clone(),
            user: self.user.clone(),
            ..Default::default()
        }
    }

    fn ps1(&self) -> String {
        format!("{}@{}:{}$ ", self.user, self.host, self.cwd)
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
            speed: 1.0,
            fg: Colour::White,
            bg: Colour::Black,
            cwd: CWD.clone(),
            host: HOST.clone(),
            user: USER.clone(),
        }
    }
}
