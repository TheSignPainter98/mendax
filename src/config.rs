// use crate::spoof::Spoof;
use lazy_static::lazy_static;
use rhai::{Array, CustomType, Engine, EvalAltResult, Map, Scope, TypeBuilder};
use thiserror::Error;

pub fn read(fname: &str, unrestricted: bool) -> Result<Lie, Box<EvalAltResult>> {
    let engine = engine(unrestricted);

    let mut scope = Scope::new();
    scope.push("lie", Lie::new(unrestricted));

    engine.run_file_with_scope(&mut scope, fname.into())?;

    Ok(scope.get_value("lie").unwrap())
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
    tale: Tale,
    allow_system: bool,
}

impl Lie {
    fn new(allow_system: bool) -> Self {
        Self {
            tale: Tale::new(),
            allow_system,
        }
    }

    pub fn tale(&self) -> &Tale {
        &self.tale
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

    fn cd(&mut self, dir: &str) {
        self.tale.push(Fib::Run {
            cmd: format!("cd {dir}"),
            result: vec![],
        });
        self.tale.push(Fib::Look {
            cwd: Some(dir.into()),
            host: None,
            user: None,
            fg: None,
            bg: None,
            speed: None,
            title: None,
        });
    }

    fn system_simple(&mut self, cmd: &str) -> Result<(), Box<EvalAltResult>> {
        self.system(cmd, cmd)
    }

    fn system(&mut self, cmd: &str, apparent_cmd: &str) -> Result<(), Box<EvalAltResult>> {
        if !self.allow_system {
            let problem = MendaxError::SystemForbidden;
            return Err(Box::new(EvalAltResult::ErrorSystem(
                        problem.to_string(),
                        problem.into(),
            )));
        }

        let apparent_cmd = apparent_cmd.into();
        let cmd = cmd.into();
        self.tale.push(Fib::System { apparent_cmd, cmd });

        Ok(())
    }

    fn look(&mut self, options: Map) -> Result<(), Box<EvalAltResult>> {
        let mut speed = None;
        let mut fg = None;
        let mut bg = None;
        let mut title = None;
        let mut cwd = None;
        let mut host = None;
        let mut user = None;

        for (k, v) in options.iter() {
            let v = v.clone();
            match k.as_str() {
                "speed" => speed = Some(v.cast()),
                "fg" => fg = Some(v.cast::<String>().as_str().try_into()?),
                "bg" => bg = Some(v.cast::<String>().as_str().try_into()?),
                "title" => title = Some(v.cast()),
                "cwd" => cwd = Some(v.cast()),
                "host" => host = Some(v.cast()),
                "user" => user = Some(v.cast()),
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

        self.tale.push(Fib::Look {
            speed,
            fg,
            bg,
            title,
            cwd,
            host,
            user,
        });

        Ok(())
    }

    fn clear(&mut self) {
        self.tale.push(Fib::Clear);
    }
}

impl CustomType for Lie {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Lie")
            .with_fn("run", Self::run_no_output)
            .with_fn("run", Self::run_short)
            .with_fn("run", Self::run_long)
            .with_fn("cd", Self::cd)
            .with_fn("system", Self::system_simple)
            .with_fn("system", Self::system)
            .with_fn("clear", Self::clear)
            .with_fn("look", Self::look);
    }
}

#[derive(Clone, Debug)]
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
    #[error("unknown field {field:?} expected one of {expected:?}")]
    UnknownField {
        field: String,
        expected: Vec<&'static str>,
    },

    #[error("system commands are forbidden at this sandbox level")]
    SystemForbidden,

    #[error("unknown colour {0}, expected one of {1:?}")]
    UnknownColour(String, &'static [&'static str]),
}

impl From<MendaxError> for EvalAltResult {
    fn from(value: MendaxError) -> Self {
        EvalAltResult::ErrorSystem(value.to_string(), Box::new(value))
    }
}

#[derive(Clone, Debug)]
pub enum Fib {
    Run {
        cmd: String,
        result: Vec<String>,
    },
    System {
        cmd: String,
        apparent_cmd: String,
    },
    Look {
        speed: Option<f64>,
        fg: Option<Colour>,
        bg: Option<Colour>,
        title: Option<String>,
        cwd: Option<String>,
        host: Option<String>,
        user: Option<String>,
    },
    Clear,
}

#[derive(Copy, Clone, Debug)]
pub enum Colour {
    Red,
    Black,
    White,
}

static COLOURS: phf::Map<&'static str, Colour> = phf::phf_map! {
    "red"   => Colour::Red,
    "black" => Colour::Black,
    "white" => Colour::White,
};

lazy_static! {
    static ref COLOUR_NAMES: Vec<&'static str> = COLOURS.keys().map(|s| (*s).into()).collect();
}

impl TryFrom<&str> for Colour {
    type Error = Box<EvalAltResult>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(c) = COLOURS.get(s) {
            Ok(*c)
        } else {
            Err(Box::new(
                    MendaxError::UnknownColour(s.to_owned().into(), &COLOUR_NAMES).into(),
            ))
        }
    }
}
