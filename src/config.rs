// use crate::spoof::Spoof;
use rhai::{CustomType, Engine, EvalAltResult, FnPtr, Map, NativeCallContext, Scope, TypeBuilder};
use thiserror::Error;

pub fn read(fname: &str, unrestricted: bool) -> Result<Lie, Box<EvalAltResult>> {
    let engine = engine(unrestricted);

    let mut scope = Scope::new();
    scope.push("lie", Lie::new(unrestricted));

    engine.run_file_with_scope(&mut scope, fname.into())?;

    let lie: Lie = scope.get_value("lie").unwrap();
    Ok(lie)
}

fn engine(unrestricted: bool) -> Engine {
    let mut engine = Engine::new();
    engine.build_type::<Lie>();

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

    fn fake_string(&mut self, cmd: &str, result: &str) {
        self.fake_strings(cmd, vec![result])
    }

    fn fake_strings(&mut self, cmd: &str, result: Vec<&str>) {
        let cmd = cmd.into();
        let result = result.into_iter().map(ToOwned::to_owned).collect();
        self.parts.push(Fib::Fake { cmd, result });
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

    fn run(&mut self, cmd: &str) -> Result<(), Box<EvalAltResult>> {
        if !self.allow_run {
            let problem = MendaxError::RunForbidden;
            return Err(Box::new(EvalAltResult::ErrorSystem(
                problem.to_string(),
                problem.into(),
            )));
        }

        self.parts.push(Fib::Run { cmd: cmd.into() });

        Ok(())
    }

    fn look(&mut self, options: Map) -> Result<(), Box<EvalAltResult>> {
        let mut speed = None;
        let mut fg = None;
        let mut bg = None;

        for (k, v) in options.iter() {
            match k.as_str() {
                "speed" => speed = Some(v.clone().cast()),
                "fg" => fg = Some(v.clone().cast::<String>().as_str().try_into()?),
                "bg" => bg = Some(v.clone().cast::<String>().as_str().try_into()?),
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
        let child = lie.child();
        let child: Lie = f.call_within_context(&ctx, (child,))?;

        lie.parts.push(Fib::Screen {
            parts: child.parts.clone(),
        });

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

    #[error("unknown colour {0}, expected one of 'black' or 'red'")]
    UnknownColour(String),
}

impl From<MendaxError> for EvalAltResult {
    fn from(value: MendaxError) -> Self {
        EvalAltResult::ErrorSystem(value.to_string(), Box::new(value))
    }
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
        fg: Option<Colour>, // TODO(kcza): get format the colours!
        bg: Option<Colour>,
    },
    Screen {
        parts: Vec<Fib>,
    },
}

#[derive(Clone, Debug)]
pub enum Colour {
    Red,
    Black,
}

impl TryFrom<&str> for Colour {
    type Error = Box<EvalAltResult>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match &s.to_lowercase()[..] {
            "red" => Ok(Self::Red),
            "black" => Ok(Self::Black),
            _ => Err(Box::new(MendaxError::UnknownColour(s.to_owned()).into())),
        }
    }
}
