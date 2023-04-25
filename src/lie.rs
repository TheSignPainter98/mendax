use crate::{fib::Fib, MendaxError};
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
    time::Duration,
};

pub fn read<P: AsRef<Path>>(fname: P, unrestricted: bool) -> Result<Lie, Box<EvalAltResult>> {
    let fname = fname.as_ref();

    let engine = engine(unrestricted);

    let mut scope = Scope::new();
    scope.push("lie", SharedLie::new(unrestricted));

    let src = get_src(fname)?;
    let ast = engine.compile_with_scope(&scope, src)?;
    engine.run_ast_with_scope(&mut scope, &ast)?;

    scope.get_value::<SharedLie>("lie").unwrap().try_into()
}

fn get_src(fname: &Path) -> Result<String, Box<EvalAltResult>> {
    let exact = fs::read_to_string(fname);
    let inferred = fs::read_to_string((fname.to_string_lossy() + ".rhai").to_string());
    if exact.is_ok() && inferred.is_ok() {
        return Err(Box::new(
            MendaxError::AmbiguousSource {
                f1: fname.to_string_lossy().to_string(),
                f2: fname.to_string_lossy().to_string() + ".rhai",
            }
            .into(),
        ));
    }
    exact.or(inferred).map_err(|e| {
        Box::new(
            MendaxError::NoSuchSource {
                stem: fname.to_owned(),
                error: Box::new(e),
            }
            .into(),
        )
    })
}

fn engine(unrestricted: bool) -> Engine {
    let mut engine = Engine::new();
    engine.build_type::<SharedLie>();

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

#[derive(Clone, Debug)]
struct SharedLie(Rc<RefCell<Lie>>);

impl From<Lie> for SharedLie {
    fn from(tale: Lie) -> Self {
        Self(Rc::new(RefCell::new(tale)))
    }
}

impl TryFrom<SharedLie> for Lie {
    type Error = Box<EvalAltResult>;

    fn try_from(shared: SharedLie) -> Result<Self, Self::Error> {
        Ok(shared
            .0
            .try_borrow()
            .map_err(|e| {
                Box::new(EvalAltResult::from(MendaxError::LieUnreadable {
                    error: Box::new(e),
                    at: None,
                }))
            })?
            .clone())
    }
}

impl SharedLie {
    fn new(allow_system: bool) -> Self {
        Self::from(Lie::new(allow_system))
    }

    fn lie(&self, ctx: &NativeCallContext) -> Result<Ref<'_, Lie>, Box<EvalAltResult>> {
        self.0.try_borrow().map_err(|e| {
            Box::new(EvalAltResult::from(MendaxError::LieUnreadable {
                error: Box::new(e),
                at: Some(ctx.position()),
            }))
        })
    }

    fn lie_mut(&self, ctx: &NativeCallContext) -> Result<RefMut<'_, Lie>, Box<EvalAltResult>> {
        self.0.try_borrow_mut().map_err(|e| {
            Box::new(EvalAltResult::from(MendaxError::LieUnwritable {
                error: Box::new(e),
                at: Some(ctx.position()),
            }))
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
        lie.lie_mut(&ctx)?.look(ctx, options)
    }

    fn tag(ctx: NativeCallContext, lie: &mut Self, name: &str) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.tag(ctx, name)
    }

    fn sleep(
        ctx: NativeCallContext,
        lie: &mut Self,
        millis: i64,
    ) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.sleep(millis as u64)
    }

    fn stop(ctx: NativeCallContext, lie: &mut Self) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.stop();
        Ok(())
    }

    fn clear(ctx: NativeCallContext, lie: &mut Self) -> Result<(), Box<EvalAltResult>> {
        lie.lie_mut(&ctx)?.clear();
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Lie {
    fibs: Vec<Fib>,
    known_tags: Rc<RefCell<HashSet<String>>>,
    allow_system: bool,
    root: bool,
}

impl Lie {
    fn new(allow_system: bool) -> Self {
        Self {
            fibs: Vec::new(),
            known_tags: Rc::new(RefCell::new(HashSet::new())),
            allow_system,
            root: true,
        }
    }

    pub fn into_fibs(self) -> Vec<Fib> {
        self.fibs
    }

    fn child(&self) -> Self {
        Self {
            fibs: vec![],
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
        self.fibs.push(Fib::Run { cmd, result });
    }

    fn show(&mut self, text: &str) {
        let text = text.into();
        self.fibs.push(Fib::Show { text });
    }

    fn cd(&mut self, dir: &str) {
        self.fibs.push(Fib::Run {
            cmd: format!("cd {dir}"),
            result: vec![],
        });
        self.fibs.push(Fib::Look {
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
        self.fibs.push(Fib::System { apparent_cmd, cmd });

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

        let child = SharedLie::from(self.child());
        f.call_within_context(&ctx, (child.clone(),))?;

        self.fibs.push(Fib::Screen {
            apparent_cmd: apparent_cmd.map(ToOwned::to_owned),
            fibs: child.lie(&ctx)?.fibs.clone(),
        });

        Ok(())
    }

    fn look(&mut self, ctx: NativeCallContext, options: Map) -> Result<(), Box<EvalAltResult>> {
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
                &mut dyn FnMut(Dynamic) -> Result<(), &'static str>,
            ); 6] = [
                ("speed", &mut |v: Dynamic| {
                    speed = Some(v.try_cast().ok_or("f64")?);
                    Ok(())
                }),
                ("title", &mut |v: Dynamic| {
                    title = Some(v.try_cast().ok_or("string")?);
                    Ok(())
                }),
                ("cwd", &mut |v: Dynamic| {
                    cwd = Some(v.try_cast().ok_or("string")?);
                    Ok(())
                }),
                ("host", &mut |v: Dynamic| {
                    host = Some(v.try_cast().ok_or("string")?);
                    Ok(())
                }),
                ("user", &mut |v: Dynamic| {
                    user = Some(v.try_cast().ok_or("string")?);
                    Ok(())
                }),
                ("final_prompt", &mut |v: Dynamic| {
                    final_prompt = Some(v.try_cast().ok_or("string")?);
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
                    let type_name = v.type_name();
                    action(v.clone()).map_err(|e| {
                        EvalAltResult::ErrorMismatchDataType(
                            e.into(),
                            type_name.into(),
                            ctx.position(),
                        )
                    })?;
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

        self.fibs.push(Fib::Look {
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
        let name = name.trim().to_string();

        match &name[..] {
            "" | "?" | "!" => {
                return Err(Box::new(MendaxError::InvalidTagName { name }.into()));
            }
            _ => {}
        }

        let new_tag = self
            .known_tags
            .try_borrow_mut()
            .map_err(|e| {
                EvalAltResult::from(MendaxError::LieUnwritable {
                    error: Box::new(e),
                    at: Some(ctx.position()),
                })
            })?
            .insert(name.clone());
        if !new_tag {
            return Err(Box::new(MendaxError::DuplicateTag { name }.into()));
        }

        self.fibs.push(Fib::Tag { name });
        Ok(())
    }

    fn sleep(&mut self, millis: u64) -> Result<(), Box<EvalAltResult>> {
        self.fibs.push(Fib::Sleep {
            duration: Duration::from_millis(millis),
        });

        Ok(())
    }

    fn stop(&mut self) {
        self.fibs.push(Fib::Stop);
    }

    fn clear(&mut self) {
        self.fibs.push(Fib::Clear);
    }
}

impl CustomType for SharedLie {
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
            .with_fn("sleep", Self::sleep)
            .with_fn("stop", Self::stop)
            .with_fn("clear", Self::clear);
    }
}

#[cfg(test)]
impl Lie {
    pub fn fibs(&self) -> &[Fib] {
        &self.fibs
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use regex::Regex;
    use std::{
        error::Error,
        fs::{self, File},
        io::Write,
    };
    use tempfile::TempDir;

    pub fn test_script(unrestricted: bool, script: &str) -> Result<Lie, Box<dyn Error>> {
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

        assert!(read(&path, false).is_ok());
        assert!(read(path.with_extension(""), false).is_ok());

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
            lie.fibs(),
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
            lie.fibs(),
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
            lie.fibs(),
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
                lie.fibs(),
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
                .fibs(),
                &[Fib::Screen {
                    apparent_cmd: None,
                    fibs: vec![Fib::System {
                        cmd: "sudo ls /root".into(),
                        apparent_cmd: Some("ls".into()),
                    }]
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
                .fibs(),
                &[Fib::Screen {
                    apparent_cmd: Some("man foo".into()),
                    fibs: vec![Fib::System {
                        cmd: "sudo ls /root".into(),
                        apparent_cmd: Some("ls".into()),
                    }]
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
            test_script(false, r#"lie.tag("foo")"#).unwrap().fibs(),
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

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.tag("!")
                "#,
            )
            .unwrap_err()
            .to_string(),
            "mendax error: tag '!' is reserved",
        );

        assert_eq!(
            test_script(
                false,
                r#"
                    lie.tag("?")
                "#,
            )
            .unwrap_err()
            .to_string(),
            "mendax error: tag '?' is reserved",
        );

        Ok(())
    }

    #[test]
    fn look() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.look(#{});"#)?.fibs(),
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

        assert_eq!(
            test_script(false, r#"lie.look(#{speed: "speedy"});"#)
                .unwrap_err()
                .to_string(),
            "Data type incorrect: string (expecting f64) (line 1, position 5)"
        );

        Ok(())
    }

    #[test]
    fn sleep() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.sleep(250)"#)?.fibs(),
            &[Fib::Sleep {
                duration: Duration::from_millis(250)
            }]
        );
        Ok(())
    }

    #[test]
    fn stop() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            test_script(false, r#"lie.stop(); lie.clear()"#)?.fibs(),
            &[Fib::Stop, Fib::Clear]
        );
        Ok(())
    }

    #[test]
    fn clear() -> Result<(), Box<dyn Error>> {
        assert_eq!(test_script(false, r#"lie.clear()"#)?.fibs(), &[Fib::Clear]);

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

        let re = Regex::new("^mendax error: ambiguous source: both .* and .* exist$").unwrap();
        assert!(re.is_match(&err_string), "unexpected error: {err_string}");

        Ok(())
    }
}
