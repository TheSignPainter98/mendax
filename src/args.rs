use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[warn(missing_docs)]
pub struct Args {
    /// The current working directory of the fake command-line user
    #[arg(long, default_value = "~")]
    dir: String,

    /// The host name of the fake command-line machine
    #[arg(long, env = "HOST", default_value = "ubuntu")]
    host: String,

    /// YAML file describing the CLI to spoof
    #[arg(value_name = "spec", default_value = "cli.yml")]
    input: String,

    /// The average time between typed characters
    #[arg(long, value_name = "ms", default_value = "45")]
    typing_interval: u32,

    /// The username of the fake command-line user
    #[arg(long, env = "USER", default_value = "ubuntu")]
    user: String,
}

impl Args {
    pub fn dir(&self) -> &str {
        &self.dir
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn typing_interval(&self) -> u32 {
        self.typing_interval
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn ps1(&self, curr_dir: Option<&str>) -> String {
        format!(
            "{}@{}:{}$ ",
            self.user(),
            self.host(),
            curr_dir.unwrap_or_else(|| self.dir())
        )
    }
}
