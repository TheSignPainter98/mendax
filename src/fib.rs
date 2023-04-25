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
        fibs: Vec<Fib>,
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
