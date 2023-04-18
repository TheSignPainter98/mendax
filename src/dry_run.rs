use crate::config::{Fib, Lie, Tale};

pub struct DryRunBuilder {
    buf: Vec<String>,
}

impl DryRunBuilder {
    fn new() -> Self {
        Self { buf: vec![] }
    }

    fn add_line<S: Into<String>>(&mut self, line: S, indent: usize) {
        if !self.buf.is_empty() {
            self.buf.push("\n".into());
        }
        self.buf.push("\t".repeat(indent));
        self.buf.push(line.into());
    }

    fn build(self) -> String {
        self.buf.concat()
    }
}

pub trait DryRun {
    fn dry_run(&self) -> String {
        let mut builder = DryRunBuilder::new();
        self.build_dry_run(&mut builder, 0);
        builder.build()
    }

    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize);
}

impl DryRun for Lie {
    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize) {
        self.tale().build_dry_run(builder, depth);
    }
}

impl DryRun for Tale {
    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize) {
        for fib in self.fibs() {
            fib.build_dry_run(builder, depth);
        }
    }
}

impl DryRun for Fib {
    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize) {
        match self {
            Self::Run { cmd, result } => {
                builder.add_line(format!("$ {cmd}"), depth);
                if !result.is_empty() {
                    for line in result {
                        builder.add_line(format!("# {line}"), depth + 1);
                    }
                }
            }
            Self::Show { text } => {
                builder.add_line(format!("# {text}"), depth);
            }
            Self::System { apparent_cmd, cmd } => {
                if let Some(apparent_cmd) = apparent_cmd {
                    builder.add_line(
                        format!("! {apparent_cmd} (secretly calls: {cmd})"),
                        depth,
                    );
                } else {
                    builder.add_line(format!("$ {cmd}"), depth);
                }
            }
            Self::Screen{ apparent_cmd, tale } => {
                if let Some(apparent_cmd) = apparent_cmd {
                    builder.add_line(format!("$ {apparent_cmd}"), depth)
                }
                builder.add_line("SCREEN", depth);
                tale.build_dry_run(builder, depth + 1);
            }
            Self::Look { speed, title, cwd, user, host, final_prompt } => {
                let mut to_change = vec![];
                if let Some(speed) = speed {
                    to_change.push(("speed", speed.to_string()));
                }
                if let Some(title) = title {
                    to_change.push(("title", title.clone()));
                }
                if let Some(cwd) = cwd {
                    to_change.push(("cwd", cwd.clone()));
                }
                if let Some(user) = user {
                    to_change.push(("user", user.clone()));
                }
                if let Some(host) = host {
                    to_change.push(("host", host.clone()));
                }
                if let Some(final_prompt) = final_prompt {
                    to_change.push(("speed", final_prompt.to_string()));
                }

                builder.add_line(format!("STYLE: {}", to_change.iter().map(|(field, value)| format!("{field}={value}")).collect::<Vec<_>>().join(", ")), depth);
            }
            Self::Clear => builder.add_line("CLEAR", depth),
        }
    }
}

#[cfg(test)]
mod test {
    // simple...
}
