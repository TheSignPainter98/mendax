// use crate::spoof::Spoof;
use rhai::{CustomType, Engine, EvalAltResult, FnPtr, Map, Scope, TypeBuilder, NativeCallContext};
use thiserror::Error;
use std::str::FromStr;

pub fn read(fname: &str, allow_run: bool) -> Result<Lie, Box<EvalAltResult>> {
    let engine = {
        let mut engine = Engine::new();
        engine.build_type::<Lie>();
        engine
    };

    let mut scope = Scope::new();
    scope.push("lie", Lie::new(allow_run));

    engine.run_file_with_scope(&mut scope, fname.into())?;

    let lie: Lie = scope.get_value("lie").unwrap();
    Ok(lie)
}

#[derive(Clone, Debug)]
pub struct Lie {
    parts: Vec<Fib>,
    allow_run: bool,
}

impl Lie {
    fn new(allow_run: bool) -> Self {
        Self {
            parts: Vec::new(),
            allow_run,
        }
    }

    fn child(&self) -> Self {
        Self::new(self.allow_run)
    }

    pub fn parts(&self) -> &Vec<Fib> {
        &self.parts
    }

    fn fake_string(&mut self, cmd: &str, result: &str) -> Result<(), Box<EvalAltResult>> {
        self.fake_strings(cmd, vec![result])
    }

    fn fake_strings(&mut self, cmd: &str, result: Vec<&str>) -> Result<(), Box<EvalAltResult>> {
        if !self.allow_run {
            let problem = MendaxError::RunForbidden;
            return Err(Box::new(EvalAltResult::ErrorSystem(
                problem.to_string(),
                problem.into(),
            )));
        }

        let cmd = cmd.into();
        let result = result.into_iter().map(ToOwned::to_owned).collect();
        self.parts.push(Fib::Fake { cmd, result });

        Ok(())
    }

    fn prompt(&mut self, options: Map) -> Result<(), Box<EvalAltResult>> {
        let mut cwd = None;
        let mut host = None;
        let mut user = None;

        for (k, v) in options.iter() {
            match k.as_str() {
                "cwd" => cwd = Some(v.clone().cast()),
                "host" => host = Some(v.clone().cast()),
                "user" => user = Some(v.clone().cast()),
                _ => {
                    let err = MendaxError::UnknownField {
                        field: k.as_str().to_owned(),
                        expected: vec!["cwd", "host", "user"],
                    };
                    return Err(Box::new(EvalAltResult::ErrorSystem(
                        err.to_string(),
                        Box::new(err),
                    )));
                }
            }
        }

        self.parts.push(Fib::Prompt { cwd, host, user });

        Ok(())
    }

    fn run(&mut self, cmd: &str) {
        self.parts.push(Fib::Run { cmd: cmd.into() })
    }

    fn look(&mut self, options: Map) -> Result<(), Box<EvalAltResult>> {
        let mut speed = None;
        let mut fg = None;
        let mut bg = None;

        for (k, v) in options.iter() {
            match k.as_str() {
                "speed" => speed = Some(v.clone().cast()),
                "fg" => fg = Some(v.clone().cast().try_into()),
                "bg" => bg = Some(v.clone().cast().try_into()),
                _ => {
                    let err = MendaxError::UnknownField {
                        field: k.as_str().to_owned(),
                        expected: vec!["speed", "fg", "bg"],
                    };
                    return Err(Box::new(EvalAltResult::ErrorSystem(
                        err.to_string(),
                        Box::new(err),
                    )));
                }
            }
        }

        self.parts.push(Fib::Look { speed, fg, bg });

        Ok(())
    }

    fn screen(ctx: NativeCallContext, lie: &mut Self, f: FnPtr) -> Result<(), Box<EvalAltResult>> {
        // println!("{:?}", ctx);
        let child = lie.child();
        let child: Lie = f.call_within_context(&ctx, (std::rc::Rc::new(std::sync::Mutex::new(child)),))?;

        lie.parts.push(Fib::Screen { parts: child.parts.clone() });

        Ok(())
    }
}

impl CustomType for Lie {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Lie")
            .with_fn("fake", Self::fake_string)
            .with_fn("fake", Self::fake_strings)
            .with_fn("run", Self::run)
            .with_fn("prompt", Self::prompt)
            .with_fn("screen", Self::screen)
            .with_fn("look", Self::look);
    }
}

#[derive(Debug, Error)]
pub enum MendaxError {
    #[error("unknown field {field:?} expected one of {expected:?}")]
    UnknownField {
        field: String,
        expected: Vec<&'static str>,
    },

    #[error("run commands are forbidden at this sandbox level")]
    RunForbidden,
}

#[derive(Clone, Debug)]
pub enum Fib {
    Fake {
        cmd: String,
        result: Vec<String>,
    },
    Prompt {
        cwd: Option<String>,
        host: Option<String>,
        user: Option<String>,
    },
    Run {
        cmd: String,
    },
    Look {
        speed: Option<f64>,
        fg: Option<String>, // TODO(kcza): get format the colours!
        bg: Option<String>,
    },
    Screen {
        parts: Vec<Fib>,
    },
}

// #[derive(strum_macros::EnumString)]
// pub enum Colour {
//     Red,
//     Black
// }
