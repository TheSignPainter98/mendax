use crossterm::style::Stylize;
// use std::collections::HashMap;
// use std::error::Error;
// use std::fmt::Display;
// use std::io::{Read, StdoutLock, Write};
// use std::{thread, time::Duration};
// use subprocess::{Exec, Redirection};

// pub trait Tell {
//     fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>>;
// }

// impl Tell for Lie {
//     fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
//         terminal::enable_raw_mode()?;
//         execute!(
//             stdout,
//             Hide,
//             DisableBlinking,
//             Clear(ClearType::All),
//             MoveTo(0, 0)
//         )?;

//         let ret = self.tale().tell(stdout, style);

//         if style.final_prompt {
//             Fib::Run {
//                 cmd: String::new(),
//                 result: vec![],
//             }
//             .tell(stdout, style)?;
//         }

//         if style.insert_newline {
//             execute!(stdout, Print("\r\n"))?;
//         }

//         execute!(stdout, EnableBlinking, Show)?;
//         terminal::disable_raw_mode()?;

//         ret
//     }
// }

// impl Tell for Tale {
//     fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
//         for fib in self.fibs() {
//             fib.tell(stdout, style)?;
//         }

//         Ok(())
//     }
// }

// impl Tell for Fib {
//     fn tell(&self, stdout: &mut StdoutLock, style: &mut Style) -> Result<(), Box<dyn Error>> {
//         match self {
//             Self::Run { cmd, result } => {
//                 if style.insert_newline {
//                     execute!(stdout, Print("\r\n"))?;
//                 }

//                 prompt(stdout, style, cmd)?;

//                 for line in result {
//                     execute!(stdout, Print(line), Print("\r\n"))?;
//                 }

//                 style.insert_newline = false;

//                 Ok(())
//             }
//             Self::Show { text } => {
//                 if style.insert_newline {
//                     execute!(stdout, Print("\r\n"))?;
//                 }

//                 execute!(stdout, Print(text), Print("\r\n"))?;
//                 style.insert_newline = false;

//                 Ok(())
//             }
//             Self::System { apparent_cmd, cmd } => {
//                 if style.insert_newline {
//                     execute!(stdout, Print("\r\n"))?;
//                 }

//                 prompt(stdout, style, apparent_cmd.as_ref().unwrap_or(cmd))?;

//                 let out = Exec::shell(cmd)
//                     .stderr(Redirection::Merge)
//                     .stream_stdout()?;
//                 let mut final_newline = false;
//                 let mut stdout_nonempty = false;
//                 for b in out.bytes() {
//                     stdout_nonempty = true;
//                     let b = b?;
//                     if b == b'\n' {
//                         final_newline = true;
//                         execute!(stdout, Print("\r\n"))?;
//                     } else {
//                         final_newline = false;
//                         stdout.write_all(&[b])?;
//                     }
//                 }
//                 if stdout_nonempty && !final_newline {
//                     execute!(
//                         stdout,
//                         SetAttribute(Attribute::Reverse),
//                         Print("%"),
//                         SetAttribute(Attribute::Reset),
//                         Print("\r\n")
//                     )?;
//                 }
//                 execute!(stdout, SetAttribute(Attribute::Reset))?;
//                 style.insert_newline = false;

//                 Ok(())
//             }
//             Self::Screen { apparent_cmd, tale } => {
//                 if style.insert_newline {
//                     execute!(stdout, Print("\r\n"))?;
//                 }

//                 if let Some(apparent_cmd) = apparent_cmd {
//                     prompt(stdout, style, apparent_cmd)?;
//                 }

//                 execute!(stdout, EnterAlternateScreen, SavePosition, MoveTo(0, 0))?;

//                 style.insert_newline = false;
//                 let ret = tale.tell(stdout, style);

//                 pause()?;

//                 execute!(stdout, RestorePosition, LeaveAlternateScreen)?;
//                 ret
//             }
//             Self::Look {
//                 speed,
//                 title,
//                 cwd,
//                 host,
//                 user,
//                 final_prompt,
//             } => {
//                 if let Some(title) = title {
//                     execute!(stdout, SetTitle(title)).unwrap();
//                 }
//                 if let Some(cwd) = cwd {
//                     style.cwd = cwd.clone();
//                 }
//                 if let Some(host) = host {
//                     style.host = host.clone();
//                 }
//                 if let Some(user) = user {
//                     style.user = user.clone();
//                 }
//                 if let Some(speed) = speed {
//                     style.speed = *speed;
//                 }
//                 if let Some(final_prompt) = final_prompt {
//                     style.final_prompt = *final_prompt;
//                 }
//                 // TODO(kcza): colour support?

//                 Ok(())
//             }
//             Self::Tag { .. } => Ok(()),
//             Self::Clear => {
//                 execute!(stdout, Clear(ClearType::All))?;
//                 style.insert_newline = false;
//                 Ok(())
//             }
//         }
//     }
// }

// fn prompt(stdout: &mut StdoutLock, style: &mut Style, cmd: &str) -> Result<(), Box<dyn Error>> {
//     execute!(stdout, Print(style.ps1()))?;
//     execute!(stdout, Show, DisableBlinking)?;
//     stdout.flush()?;

//     pause()?;
//     if !cmd.is_empty() {
//         style.fake_type(stdout, cmd.chars().collect::<Vec<_>>().as_slice())?;
//         pause()?;
//     }
//     execute!(stdout, Hide, EnableBlinking)?;
//     execute!(stdout, Print("\r\n"))?;
//     stdout.flush()?;

//     Ok(())
// }
