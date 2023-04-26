use std::{
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    path::Path,
    process::ExitCode,
};

const EXAMPLE: &str = r#"
lie.look(#{ title: "legit demo" });

lie.run("echo Hello, world", "Hello, world\n");
lie.run("echo 'All of this is fake'", "All of this is fake\n");

lie.cd("~");

lie.run("ls -A1", [
    ".bash_history\n",
    ".bashrc\n",
    ".cargo\n",
    ".rustup\n",
    ".vimrc\n",
    ".zshrc\n",
    "Desktop\n",
    "Documents\n",
    "Downloads\n",
    "snap\n",
]);
"#;

pub fn init(fname: &Path) -> ExitCode {
    match init_example(fname) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn init_example(fname: &Path) -> io::Result<()> {
    let f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(fname)?;
    let mut w = BufWriter::new(f);

    write!(w, "{}", &EXAMPLE[1..])?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use super::*;
    use crate::lie;
    use tempdir::TempDir;

    #[test]
    fn valid() -> Result<(), Box<dyn Error>> {
        let tmp_dir = TempDir::new("mendax_test_valid")?;

        let example_lie = tmp_dir.path().join("foo.rhai");
        init_example(&example_lie)?;

        let result = lie::read(example_lie.to_string_lossy().as_ref(), false);
        assert!(result.is_ok(), "unexpected error: {}", result.unwrap_err());

        Ok(())
    }
}
