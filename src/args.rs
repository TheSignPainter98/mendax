use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[warn(missing_docs)]
pub struct Args {
    /// Initialise new mendax project
    #[arg(long)]
    init: bool,

    /// YAML file describing the CLI to spoof
    #[arg(value_name = "spec", default_value_t = String::from("lie.rhai"))]
    spec: String,

    /// Allow exectution of arbitrary shell commands
    #[arg(long = "unsafe")]
    unrestricted: bool,
}

impl Args {
    pub fn init(&self) -> bool {
        self.init
    }

    pub fn input(&self) -> &str {
        &self.spec
    }

    pub fn unrestricted(&self) -> bool {
        self.unrestricted
    }
}
