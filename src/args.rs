use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[warn(missing_docs)]
pub struct Args {
    /// Initialise new mendax project
    #[arg(long)]
    init: bool,

    /// Rhai scriptlet describing the CLI to spoof, file extension optional
    #[arg(value_name = "spec", default_value_t = String::from("lie.rhai"))]
    spec: String,

    /// Allow execution of arbitrary shell commands
    #[arg(long = "unsafe")]
    unrestricted: bool,

    /// Output all commands which would be run
    #[arg(long)]
    dry_run: bool,
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

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
}
