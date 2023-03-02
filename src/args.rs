use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
#[warn(missing_docs)]
pub struct Args {
    /// Initialise new mendax project
    #[arg()]
    init: Option<Init>,

    /// YAML file describing the CLI to spoof [default: cli.yml]
    #[arg(long, value_name = "spec", default_value_t = String::from("lie.rhai"))]
    spec: String,

    /// Allow exectution of arbitrary shell commands
    #[arg(long = "unsafe")]
    unrestricted: bool,
}

impl Args {
    pub fn init(&self) -> &Option<Init> {
        &self.init
    }

    pub fn input(&self) -> &str {
        &self.spec
    }

    pub fn unrestricted(&self) -> bool {
        self.unrestricted
    }
}

#[derive(ValueEnum, Clone)]
pub enum Init {
    Init
}
