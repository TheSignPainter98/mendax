# Mendax

A CLI spoofer—like [asciinema][asciinema] but in real-time.

Specify a sequence of inputs and outputs, printed one at a time as you press enter.

## Table of Contents

<!-- vim-markdown-toc GFM -->

* [Installation instructions](#installation-instructions)
* [Usage](#usage)
* [Input format](#input-format)
    * [Input actions](#input-actions)
    * [Output actions](#output-actions)
* [Known Issues](#known-issues)
    * [`$HOST` not auto-detected](#host-not-auto-detected)
* [Author, License and Name](#author-license-and-name)

<!-- vim-markdown-toc -->

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

## Input format

The sequence of actions to be shown is given in a YAML file.
These actions are either input actions or output actions.

```yaml
- cmd: gcc --version
- print: |
    gcc (Ubuntu 11.3.0-1ubuntu1~22.04) 11.3.0
    Copyright (C) 2021 Free Software Foundation, Inc.
    This is free software; see the source for copying conditions.  There is NO
    warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
```

This will type out `gcc --version` into the fake command prompt and output `gcc (Ubuntu 11.3.0-...`.
Each time the output stops (for example to allow the user to explain something), any key may be pressed to continue the demonstration.

### Input actions

An input action will show a fake terminal prompt, pause for the user to press a button, then write out the given command.
It looks like the following:

```yaml
- cmd: juju status      # Mandatory: the command to type out
  prompt: '$ '          # Optional: override default bash-style prompt
  dir: 'Documents/juju' # Optional: override default directory in default prompt
```

### Output actions

An output action will print something verbatim onto the screen.
This has three forms: printing a block, printing a sequence of lines, or showing a screen

To print a block of text, specify a string to print:

```yaml
- print: |
    Model       Controller          Cloud/Region        Version  SLA          Timestamp
    controller  microk8s-localhost  microk8s/localhost  2.9.37   unsupported  11:19:55Z
```

To print text line by line (for example to simulate the output as a program computes values), specify a list of strings to print:

```yaml
- print:
  - Model       Controller          Cloud/Region        Version  SLA          Timestamp
  - controller  microk8s-localhost  microk8s/localhost  2.9.37   unsupported  11:19:55Z
  speed: snail # Optional: override default print frequency
```

## Known Issues

### `$HOST` not auto-detected

The default promps reads the host name from the `$HOST` environment variable, however some systems may not expose the host name by default.
Depending on your shell, running `export HOST` before `mendax` should fix this.

## Author, License and Name

This project is maintained by Ed Jones and is licensed under the GNU General Public License version 3.

The name ‘mendax’ is a Latin word which roughly translates to ‘narrator,’ ‘storyteller,’ or ‘habitual liar.’

[asciinema]: https://asciinema.org/
