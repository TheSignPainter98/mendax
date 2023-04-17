use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
#[warn(missing_docs)]
pub struct Args {
    /// Initialise new mendax project
    #[arg()]
    init: Option<Init>,

    /// YAML file describing the CLI to spoof
    #[arg(long, value_name = "spec", default_value_t = String::from("lie.rhai"))]
    spec: String,

    /// Allow exectution of arbitrary shell commands
    #[arg(long = "unsafe")]
    unrestricted: bool,

    /// Output all commands which would be run
    #[arg(long)]
    dry_run: bool,
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

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
}

#[derive(ValueEnum, Clone)]
pub enum Init {
    Init,
}
