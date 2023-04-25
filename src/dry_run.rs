use crate::{fib::Fib, lie::Lie};

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
        self.buf.push("    ".repeat(indent));
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
        self.fibs().build_dry_run(builder, depth);
    }
}

impl DryRun for &[Fib] {
    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize) {
        self.iter()
            .for_each(|fib| fib.build_dry_run(builder, depth));
    }
}

impl DryRun for Fib {
    fn build_dry_run(&self, builder: &mut DryRunBuilder, depth: usize) {
        match self {
            Self::Run { cmd, result } => {
                builder.add_line(format!("$ {cmd}"), depth);
                if !result.is_empty() {
                    for line in result {
                        builder.add_line(format!("# {line}"), depth);
                    }
                }
            }
            Self::Show { text } => {
                builder.add_line(format!("# {text}"), depth);
            }
            Self::System { apparent_cmd, cmd } => {
                if let Some(apparent_cmd) = apparent_cmd {
                    builder.add_line(format!("! {apparent_cmd} (secretly calls: {cmd})"), depth);
                } else {
                    builder.add_line(format!("! {cmd}"), depth);
                }
            }
            Self::Screen { apparent_cmd, fibs } => {
                if let Some(apparent_cmd) = apparent_cmd {
                    builder.add_line(format!("$ {apparent_cmd}"), depth)
                }
                builder.add_line("(screen)", depth);
                fibs.as_slice().build_dry_run(builder, depth + 1);
            }
            Self::Look {
                speed,
                title,
                cwd,
                user,
                host,
                final_prompt,
            } => {
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

                builder.add_line(
                    format!(
                        "(look: {})",
                        to_change
                            .iter()
                            .map(|(field, value)| format!("{field}={value}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    depth,
                );
            }
            Self::Tag { name } => builder.add_line(format!("(tag) {name}"), depth),
            Self::Sleep { duration } => builder.add_line(
                format!(
                    "(sleep) {}",
                    pretty_duration::pretty_duration(duration, None)
                ),
                depth,
            ),
            Self::Stop => builder.add_line("(STOP)", depth),
            Self::Enter { msg } => builder.add_line(format!("(enter) {msg}"), depth),
            Self::Clear => builder.add_line("(clear)", depth),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lie;
    use indoc::indoc;

    #[test]
    fn all() {
        assert_eq!(
            lie::test::test_script(
                true,
                r#"
                    fn populate(lie) {
                        lie.run("echo foo");
                        lie.run("echo asdf", "asdf");
                        lie.run("echo asdf", ["a", "s", "d", "f"]);
                        lie.show("foo");
                        lie.cd("/root");
                        lie.system("ls");
                        lie.system("ls", "dir");
                        lie.sleep(100);
                        lie.stop();
                        lie.enter("asdf");
                    }
                    populate(lie);
                    lie.screen(|lie| {
                        populate(lie);
                    });
                    lie.screen("man foo", |lie| {
                        populate(lie);
                    });
                "#
            )
            .unwrap()
            .dry_run(),
            indoc! {r#"
                $ echo foo
                $ echo asdf
                # asdf
                $ echo asdf
                # a
                # s
                # d
                # f
                # foo
                $ cd /root
                (look: cwd=/root)
                ! ls
                ! ls (secretly calls: dir)
                (sleep) 100ms
                (STOP)
                (enter) asdf
                (screen)
                    $ echo foo
                    $ echo asdf
                    # asdf
                    $ echo asdf
                    # a
                    # s
                    # d
                    # f
                    # foo
                    $ cd /root
                    (look: cwd=/root)
                    ! ls
                    ! ls (secretly calls: dir)
                    (sleep) 100ms
                    (STOP)
                    (enter) asdf
                $ man foo
                (screen)
                    $ echo foo
                    $ echo asdf
                    # asdf
                    $ echo asdf
                    # a
                    # s
                    # d
                    # f
                    # foo
                    $ cd /root
                    (look: cwd=/root)
                    ! ls
                    ! ls (secretly calls: dir)
                    (sleep) 100ms
                    (STOP)
                    (enter) asdf
            "#}
            .trim(),
        );
    }
}
