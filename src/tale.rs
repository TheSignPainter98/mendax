#[derive(Clone, Debug, PartialEq)]
pub struct Tale(Vec<Fib>);

impl Tale {
    pub fn new(fibs: Vec<Fib>) -> Self {
        Self(fibs)
    }

    pub fn push(&mut self, fib: Fib) {
        self.0.push(fib)
    }

    pub fn into_fibs(self) -> Vec<Fib> {
        self.0
    }
}

impl Default for Tale {
    fn default() -> Self {
        Tale::new(vec![])
    }
}

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
        tale: Tale,
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
