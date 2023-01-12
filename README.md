# Mendax

A CLI spoofer—like [asciinema][asciinema] but in real-time.

Specify a sequence of inputs and outputs, printed one at a time as you press enter.

## Installation instructions

```bash
git clone https://github.com/TheSignPainter98/mendax
cd mendax
cargo install --path .
sudo install -m755 ./target/release/mendax /usr/bin/mendax
```

## Usage

```bash
mendax init # make an example in the default location
mendax # run with the default file name
```

The fake command-prompt shown can be modified using options, see `mendax --help` for more information.

[asciinema]: https://asciinema.org/
