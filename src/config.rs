use rhai::{
    Array, CustomType, Dynamic, Engine, EvalAltResult, FnPtr, Map, NativeCallContext, Scope,
    TypeBuilder,
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    fs,
    path::Path,
    rc::Rc,
};
use thiserror::Error;

pub fn read<P: AsRef<Path>>(fname: P, unrestricted: bool) -> Result<Lie, Box<EvalAltResult>> {
    let fname = fname.as_ref();

    let engine = engine(unrestricted);

    let mut scope = Scope::new();
    scope.push("lie", SharedLieBuilder::new(unrestricted));

    let src = get_src(fname)?;
    let ast = engine.compile_with_scope(&scope, src)?;
    engine.run_ast_with_scope(&mut scope, &ast)?;

    scope.get_value::<SharedLieBuilder>("lie").unwrap().build()
}

fn get_src(fname: &Path) -> Result<String, Box<EvalAltResult>> {
    let exact = fs::read_to_string(fname);
    let inferred = fs::read_to_string((fname.to_string_lossy() + ".rhai").to_string());
    if exact.is_ok() && inferred.is_ok() {
        return Err(Box::new(EvalAltResult::ErrorSystem(
            "failed to load spec".into(),
            Box::new(MendaxError::AmbiguousInput {
                f1: fname.to_string_lossy().to_string(),
                f2: fname.to_string_lossy().to_string() + ".rhai",
            }),
        )));
    }
    exact.or(inferred).map_err(|e| {
        Box::new(EvalAltResult::ErrorSystem(
            format!(
                "could not read file '{}' or '{}.rhai'",
                fname.display(),
                fname.display()
            ),
            Box::new(e),
        ))
    })
}

fn engine(unrestricted: bool) -> Engine {
    let mut engine = Engine::new();
    engine.build_type::<SharedLieBuilder>();

    if !unrestricted {
        engine.set_max_array_size(1000);
        engine.set_max_call_levels(100);
        engine.set_max_expr_depths(100, 100);
        engine.set_max_map_size(1000);
        engine.set_max_operations(10000);
        engine.set_max_string_size(15000);
    }

    engine.set_strict_variables(true);
    engine.set_fail_on_invalid_map_property(true);

    engine
}

#[derive(Debug)]
pub struct Lie {
    tale: Tale,
}

impl Lie {
    fn new(tale: Tale) -> Self {
        Self { tale }
    }

    pub fn tale(&self) -> &Tale {
        &self.tale
    }
}

#[derive(Clone, Debug)]
struct SharedLieBuilder(Rc<RefCell<LieBuilder>>);

impl From<LieBuilder> for SharedLieBuilder {
    fn from(lie: LieBuilder) -> Self {
        Self(Rc::new(RefCell::new(lie)))
    }
}

impl SharedLieBuilder {
    fn new(allow_system: bool) -> Self {
        Self::from(LieBuilder::new(allow_system))
    }

    fn build(self) -> Result<Lie, Box<EvalAltResult>> {
        Ok(Lie::new(
            self.0
                .try_borrow()
                .map_err(|e| EvalAltResult::ErrorSystem("lie in use".into(), Box::new(e)))?
                .tale
                .clone(),
        ))
    }

    fn lie(&self, ctx: &NativeCallContext) -> Result<Ref<'_, LieBuilder>, Box<EvalAltResult>> {
        self.0.try_borrow().map_err(|e| {
            Box::new(EvalAltResult::ErrorDataRace(
                format!("failed to read lie: {e}"),
                ctx.position(),
            ))
        })
    }

    fn lie_mut(
        &self,
        ctx: &NativeCallContext,
    ) -> Result<RefMut<'_, LieBuilder>, Box<EvalAltResult>> {
        self.0.try_borrow_mut().map_err(|e| {
            Box::new(EvalAltResult::ErrorDataRace(
                format!("failed to read lie: {e}"),
                ctx.position(),
            ))
        })
    }

    fn run_no_output(
        ctx: NativeCallContext,
        lie: &mut Self,
        cmd: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.run_no_output(cmd);
        Ok(())
    }

    fn run_short(
        ctx: NativeCallContext,
        lie: &mut Self,
        cmd: &str,
        result: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.run_short(cmd, result);
        Ok(())
    }

    fn run_long(
        ctx: NativeCallContext,
        lie: &mut Self,
        cmd: &str,
        result: Array,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.run_long(cmd, result)
    }

    fn show(ctx: NativeCallContext, lie: &mut Self, text: &str) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.show(text);
        Ok(())
    }

    fn cd(ctx: NativeCallContext, lie: &mut Self, dir: &str) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.cd(dir);
        Ok(())
    }

    fn system_simple(
        ctx: NativeCallContext,
        lie: &mut Self,
        cmd: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.system_simple(cmd)
    }

    fn system(
        ctx: NativeCallContext,
        lie: &mut Self,
        apparent_cmd: &str,
        cmd: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.system(Some(apparent_cmd), cmd)
    }

    fn screen_simple(
        ctx: NativeCallContext,
        lie: &mut Self,
        f: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.screen_simple(ctx, f)
    }

    fn screen(
        ctx: NativeCallContext,
        lie: &mut Self,
        apparent_cmd: &str,
        f: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.screen(ctx, Some(apparent_cmd), f)
    }

    fn look(
        ctx: NativeCallContext,
        lie: &mut Self,
        options: Map,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.look(options)
    }

    fn tag(ctx: NativeCallContext, lie: &mut Self, name: &str) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.tag(ctx, name)
    }

    fn clear(ctx: NativeCallContext, lie: &mut Self) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.clear();
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct LieBuilder {
    tale: Tale,
    known_tags: Rc<RefCell<HashSet<String>>>,
    allow_system: bool,
    root: bool,
}

impl LieBuilder {
    fn new(allow_system: bool) -> Self {
        Self {
            tale: Tale::new(),
            known_tags: Rc::new(RefCell::new(HashSet::new())),
            allow_system,
            root: true,
        }
    }

    fn child(&self) -> Self {
        Self {
            tale: Tale::new(),
            known_tags: self.known_tags.clone(),
            allow_system: self.allow_system,
            root: false,
        }
    }

    fn run_no_output(&mut self, cmd: &str) {
        self.run(cmd, vec![])
    }

    fn run_short(&mut self, cmd: &str, result: &str) {
        self.run(cmd, vec![result.into()])
    }

    fn run_long(&mut self, cmd: &str, result: Array) -> Result<(), Box<EvalAltResult>> {
        self.run(cmd, result.into_iter().map(|l| l.cast()).collect());
        Ok(())
    }

    fn run(&mut self, cmd: &str, result: Vec<String>) {
        let cmd = cmd.into();
        self.tale.push(Fib::Run { cmd, result });
    }

    fn show(&mut self, text: &str) {
        let text = text.into();
        self.tale.push(Fib::Show { text });
    }

    fn cd(&mut self, dir: &str) {
        self.tale.push(Fib::Run {
            cmd: format!("cd {dir}"),
            result: vec![],
        });
        self.tale.push(Fib::Look {
            cwd: Some(dir.into()),
            host: None,
            user: None,
            speed: None,
            title: None,
            final_prompt: None,
        });
    }

    fn system_simple(&mut self, cmd: &str) -> Result<(), Box<EvalAltResult>> {
        self.system(None, cmd)
    }

    fn system(&mut self, apparent_cmd: Option<&str>, cmd: &str) -> Result<(), Box<EvalAltResult>> {
        if !self.allow_system {
            return Err(Box::new(MendaxError::SystemForbidden.into()));
        }

        let apparent_cmd = apparent_cmd.map(ToOwned::to_owned);
        let cmd = cmd.into();
        self.tale.push(Fib::System { apparent_cmd, cmd });

        Ok(())
    }

    fn screen_simple(
        &mut self,
        ctx: NativeCallContext,
        f: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        self.screen(ctx, None, f)
    }

    fn screen(
        &mut self,
        ctx: NativeCallContext,
        apparent_cmd: Option<&str>,
        f: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        if !self.root {
            return Err(Box::new(MendaxError::NestedScreens.into()));
        }

        let child = SharedLieBuilder::from(self.child());
        f.call_within_context(&ctx, (child.clone(),))?;

        self.tale.push(Fib::Screen {
            apparent_cmd: apparent_cmd.map(ToOwned::to_owned),
            tale: child.lie(&ctx)?.tale.clone(),
        });

        Ok(())
    }

    fn look(&mut self, options: Map) -> Result<(), Box<EvalAltResult>> {
        let mut speed = None;
        let mut title = None;
        let mut cwd = None;
        let mut host = None;
        let mut user = None;
        let mut final_prompt = None;

        {
            #[allow(clippy::type_complexity)]
            let mut action_list: [(
                &str,
                &mut dyn FnMut(Dynamic) -> Result<(), Box<EvalAltResult>>,
            ); 6] = [
                ("speed", &mut |v: Dynamic| {
                    speed = Some(v.cast());
                    Ok(())
                }),
                ("title", &mut |v: Dynamic| {
                    title = Some(v.cast());
                    Ok(())
                }),
                ("cwd", &mut |v: Dynamic| {
                    cwd = Some(v.cast());
                    Ok(())
                }),
                ("host", &mut |v: Dynamic| {
                    host = Some(v.cast());
                    Ok(())
                }),
                ("user", &mut |v: Dynamic| {
                    user = Some(v.cast());
                    Ok(())
                }),
                ("final_prompt", &mut |v: Dynamic| {
                    final_prompt = Some(v.cast());
                    Ok(())
                }),
            ];

            for (k, v) in options.iter() {
                let mut found = false;
                let k = k.as_str();
                'actions: for (name, action) in &mut action_list {
                    if k != *name {
                        continue;
                    }
                    action(v.clone())?;
                    found = true;
                    break 'actions;
                }

                if !found {
                    return Err(Box::new(
                        MendaxError::UnknownField {
                            field: k.to_owned(),
                            expected: {
                                let mut expected: Vec<_> =
                                    action_list.iter().map(|(k, _)| k.to_owned()).collect();
                                expected.sort();
                                expected
                            },
                        }
                        .into(),
                    ));
                }
            }
        }

        self.tale.push(Fib::Look {
            speed,
            title,
            cwd,
            host,
            user,
            final_prompt,
        });

        Ok(())
    }

    fn tag(&mut self, ctx: NativeCallContext, name: &str) -> Result<(), Box<EvalAltResult>> {
        let name: String = name.into();

        let new_tag = self
            .known_tags
            .try_borrow_mut()
            .map_err(|e| {
                Box::new(EvalAltResult::ErrorDataRace(
                    format!("failed to read lie: {e}"),
                    ctx.position(),
                ))
            })?
            .insert(name.clone());
        if !new_tag {
            return Err(Box::new(MendaxError::DuplicateTag { name }.into()));
        }

        self.tale.push(Fib::Tag { name });
        Ok(())
    }

    fn clear(&mut self) {
        self.tale.push(Fib::Clear);
    }
}

impl CustomType for SharedLieBuilder {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Lie")
            .with_fn("run", Self::run_no_output)
            .with_fn("run", Self::run_short)
            .with_fn("run", Self::run_long)
            .with_fn("show", Self::show)
            .with_fn("cd", Self::cd)
            .with_fn("system", Self::system_simple)
            .with_fn("system", Self::system)
            .with_fn("screen", Self::screen_simple)
            .with_fn("screen", Self::screen)
            .with_fn("look", Self::look)
            .with_fn("tag", Self::tag)
            .with_fn("clear", Self::clear);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tale(Vec<Fib>);

impl Tale {
    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, fib: Fib) {
        self.0.push(fib)
    }

    pub fn fibs(&self) -> &Vec<Fib> {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum MendaxError {
    #[error("unknown field {field:?}, expected one of: {}", .expected.join(", "))]
    UnknownField {
        field: String,
        expected: Vec<&'static str>,
    },

    #[error("system calls are forbidden at this sandbox level")]
    SystemForbidden,

    #[error("unknown colour {0:?}, expected one of: {}", .1.join(", "))]
    UnknownColour(String, &'static [&'static str]),

    #[error("keyboard interrupt")]
    KeyboardInterrupt,

    #[error("cannot nest screens")]
    NestedScreens,

    #[error("both {f1} and {f2} exist")]
    AmbiguousInput { f1: String, f2: String },

    #[error("tag '{name}' defined multiple times")]
    DuplicateTag { name: String },
}

impl From<MendaxError> for EvalAltResult {
    fn from(value: MendaxError) -> Self {
        EvalAltResult::ErrorSystem("mendax error".into(), Box::new(value))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fib {
    Run {
        cmd: String,
        result: Vec<String>,
    },
    Show {
        text: String,
    },
    System {
        apparent_cmd: Option<String>,
        cmd: String,
    },
    Screen {
        apparent_cmd: Option<String>,
        tale: Tale,
    },
    Look {
        speed: Option<f64>,
        title: Option<String>,
        cwd: Option<String>,
        user: Option<String>,
        host: Option<String>,
        final_prompt: Option<bool>,
    },
    Tag {
        name: String,
    },
    Clear,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config;
    use regex::Regex;
    use std::{
        error::Error,
        fs::{self, File},
        io::Write,
    };
    use tempfile::TempDir;

    fn test_script(unrestricted: bool, script: &str) -> Result<Lie, Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let lie_path = &dir.path().join("test-lie.rhai");

        fs::write(lie_path, script)?;

        read(lie_path.as_os_str().to_str().unwrap(), unrestricted).map_err(|e| e.into())
    }

    #[test]
    fn file_extension_optional() -> Result<(), Box<dyn Error>> {
        let tmpdir = TempDir::new()?;
        let path = tmpdir.path().join("tester.rhai");
        let mut file = File::create(&path)?;
        file.write_all(r#"lie.show("hello");"#.as_bytes())?;

        assert!(config::read(&path, false).is_ok());
        assert!(config::read(path.with_extension(""), false).is_ok());

        Ok(())
    }

    #[test]
    fn run() -> Result<(), Box<dyn Error>> {
        let lie = test_script(
            false,
            r#"
                lie.run("foo");
                lie.run("bar", "qwer");
                lie.run("baz", ["asdf", "fdsa"]);
            "#,
        )?;

        assert_eq!(
            lie.tale().fibs(),
            &[
                Fib::Run {
                    cmd: "foo".into(),
                    result: vec![]
                },
                Fib::Run {
                    cmd: "bar".into(),
                    result: vec!["qwer".into()]
                },
                Fib::Run {
                    cmd: "baz".into(),
                    result: vec!["asdf".into(), "fdsa".into()]
                },
            ]
        );

        Ok(())
    }

    #[test]
    fn show() -> Result<(), Box<dyn Error>> {
        let lie = test_script(
            true,
            r#"
                lie.show("foobar");
            "#,
        )?;

        assert_eq!(
            lie.tale().fibs(),
            &[Fib::Show {
                text: "foobar".into()
            }]
        );

        Ok(())
    }

    #[test]
    fn cd() -> Result<(), Box<dyn Error>> {
        let lie = test_script(
            true,
            r#"
                lie.cd("/foo/bar");
            "#,
        )?;

        assert_eq!(
            lie.tale().fibs(),
            &[
                Fib::Run {
                    cmd: "cd /foo/bar".into(),
                    result: vec![],
                },
                Fib::Look {
                    cwd: Some("/foo/bar".into()),
                    speed: None,
                    title: None,
                    user: None,
                    host: None,
                    final_prompt: None,
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn system() -> Result<(), Box<dyn Error>> {
        {
            let lie = test_script(
                true,
                r#"
                    lie.system("ls");
                    lie.system("la", "ls -Al");
                "#,
            )?;

            assert_eq!(
                lie.tale().fibs(),
                &[
                    Fib::System {
                        cmd: "ls".into(),
                        apparent_cmd: None,
                    },
                    Fib::System {
                        cmd: "ls -Al".into(),
                        apparent_cmd: Some("la".into()),
                    },
                ]
            );
        }

        {
            match test_script(false, r#"lie.system("foo");"#) {
                Err(e) => assert_eq!(
                    e.to_string(),
                    "mendax error: system calls are forbidden at this sandbox level"
                ),
                _ => assert!(false, "system was allowed"),
            }
        }

        Ok(())
    }

    #[test]
    fn screen() -> Result<(), Box<dyn Error>> {
        {
            assert_eq!(
                test_script(
                    true,
                    r#"
                        lie.screen(|lie| {
                            lie.system("ls", "sudo ls /root");
                        });
                    "#,
                )?
                .tale()
                .fibs(),
                &[Fib::Screen {
                    apparent_cmd: None,
                    tale: Tale(vec![Fib::System {
                        cmd: "sudo ls /root".into(),
                        apparent_cmd: Some("ls".into()),
                    }])
                }]
            );

            assert_eq!(
                test_script(
                    false,
                    r#"
                        lie.screen(|lie| {
                            lie.system("ls", "sudo ls /root");
                        });
                    "#,
                )
                .unwrap_err()
                .to_string(),
                "mendax error: system calls are forbidden at this sandbox level",
            );
        }

        {
            assert_eq!(
                test_script(
                    true,
                    r#"
                        lie.screen("man foo", |lie| {
                            lie.system("ls", "sudo ls /root");
                        });
                    "#,
                )?
                .tale()
                .fibs(),
                &[Fib::Screen {
                    apparent_cmd: Some("man foo".into()),
                    tale: Tale(vec![Fib::System {
                        cmd: "sudo ls /root".into(),
                        apparent_cmd: Some("ls".into()),
                    }])
                }]
            );

            assert_eq!(
                test_script(
                    false,
                    r#"
                        lie.screen("man foo", |lie| {
                            lie.system("ls", "sudo ls /root");
                        });
                    "#,
                )
                .unwrap_err()
                .to_string(),
                "mendax error: system calls are forbidden at this sandbox level",
            );
        }

        Ok(())
    }

    #[test]
    fn tag() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.tag("foo")"#)
                .unwrap()
                .tale()
                .fibs(),
            &[Fib::Tag { name: "foo".into() }],
        );

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.tag("foo");
                    lie.tag("foo");
                "#,
            )
            .unwrap_err()
            .to_string(),
            "mendax error: tag 'foo' defined multiple times",
        );

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.tag("foo");
                    lie.screen(|lie| {
                        lie.tag("foo");
                    });
                "#,
            )
            .unwrap_err()
            .to_string(),
            "mendax error: tag 'foo' defined multiple times",
        );

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.screen(|lie| {
                        lie.tag("foo");
                    });
                    lie.tag("foo");
                "#,
            )
            .unwrap_err()
            .to_string(),
            "mendax error: tag 'foo' defined multiple times",
        );

        Ok(())
    }

    #[test]
    fn look() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.look(#{});"#)?.tale().fibs(),
            &[Fib::Look {
                speed: None,
                title: None,
                cwd: None,
                host: None,
                user: None,
                final_prompt: None,
            }],
        );

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.look(#{
                        speed: 100.0,
                        title: "on the origin of electric toasters",
                        cwd: "~/toast",
                        user: "methos",
                        host: "gaia",
                        final_prompt: false,
                    });
                "#
            )?
            .tale()
            .fibs(),
            &[Fib::Look {
                speed: Some(100.0),
                title: Some("on the origin of electric toasters".into()),
                cwd: Some("~/toast".into()),
                host: Some("gaia".into()),
                user: Some("methos".into()),
                final_prompt: Some(false),
            }]
        );

        Ok(())
    }

    #[test]
    fn clear() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.clear()"#)?.tale().fibs(),
            &[Fib::Clear]
        );

        Ok(())
    }

    #[test]
    fn ambiguous_files_rejected() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let lie_path_exact = &dir.path().join("test-lie");
        let lie_path_inferred = &dir.path().join("test-lie.rhai");

        fs::write(lie_path_exact, r#"lie.show("asdf")"#)?;
        fs::write(lie_path_inferred, r#"lie.show("asdf")"#)?;

        let err_string = read(lie_path_exact.as_os_str().to_str().unwrap(), false)
            .unwrap_err()
            .to_string();

        let re = Regex::new("^failed to load spec: both .* and .* exist$").unwrap();
        assert!(re.is_match(&err_string), "unexpected error: {err_string}");

        Ok(())
    }
}
