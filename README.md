# Mendax

A CLI spoofer—like [asciinema][asciinema] but in real-time.
Specify inputs to fake, outputs to fake, all of which may be checked before running.

## Table of Contents

<!-- vim-markdown-toc GFM -->

* [Why?](#why)
* [Installation](#installation)
* [Not sure where to start?](#not-sure-where-to-start)
* [Writing the lie](#writing-the-lie)
* [Input format](#input-format)
    * [Input actions](#input-actions)
    * [Output actions](#output-actions)
* [Author, License and Name](#author-license-and-name)

<!-- vim-markdown-toc -->

## Why?

In my experience, a demo of a product is _far_ more likely to break than production code. If code is in production, it’s likely gone through many layers of testing and checking so there’s a pretty good change it does what it says on the tin. If code is used in demos, suddenly, it’s a completly different matter. Perhaps that minor change you made just fixing ‘that one last thing’ before the presentation turns out to break something critical. Perhaps your internet connection decides it just doesn’t want to work anymore, despite working literally ten minutes earlier. Suddenly, due factors which may be beyond your control, you’re now awkwardly filling time trying to fix code, questioning both the program itself and your life choices which led up to this moment.

Well no more.

Mendax offers:

- **Sandboxed spoofing---**unless you specifically allow it, mendax runs in a complete sandbox, unable to read or alter the host system’s state
- **Dry running---**demos can be checked ahead of time
- **Time travel---**tag important moments in the demo so you can jump between them at showtime

Oh and one more thing, creating asciinema and getting annoyed at your typing speed and spelling mistakes? (I often do)
All these problems can go away just by running---

```bash
asciinema rec -c mendax
```
And mendax will run your demo for you, all you have to do is time the prompts as you like.

## Installation

There is a [snap][snap]! Simply install it by running

```bash
sudo snap install mendax
```

To build and install from the source, use

```bash
git clone https://github.com/TheSignPainter98/mendax
cd mendax
cargo install --path .
sudo install -m755 ./target/release/mendax /usr/bin/mendax
```

## Not sure where to start?

Run `mendax --init` to create a new example lie, then run `mendax` for a quick demo.

## Writing the lie

The lie to be told by `mendax` is specified in the form of a [Rhai][rhai] scriptlet.
Declarations are made by calling methods on the given `lie`.

To pretend to run a command `foo`, use `lie.run`.
This has three forms: the first will pretend to run `foo` printing no output, the second will spoof `foo` printing `bar` to its stdout and the third will do the same but with `bar` and `baz` being printed with a (by default) small delay between.
```rhai
lie.run("foo");
lie.run("foo", "bar");
lie.run("foo", ["bar", "baz"]);
```

To pretend to `cd` into a given directory (and hence update the prompt), use `lie.cd`.
```rhai
lie.cd(“/dir”);
```

To show some text, use `lie.show`.
```rhai
lie.show("foo");
```

To pretend to type a string, use `lie.enter`.
This is similar to `lie.run` but without the terminal prompt being shown.

To open another screen and run more lie commands in there use `lie.screen`.
This method has two forms, where the last argument is always a closure which is passed a lie.
The first argument is optionally a command to pretend to run.
```rhai
lie.screen(|lie| {
    lie.show("I’m in another screen!");
});
lie.screen("man foo", |lie| {
    lie.show("I’m in another screen, pretending to be the manual page for foo");
});
```

To tag a point in the lie to be returned to later (by pressing `/` when output pauses, use `lie.tag`.
```rhai
lie.tag("something-interesting");
```

To clear the screen use `lie.clear`.
This takes no arguments.
```rhai
lie.clear()
```

To stop execution early, use `lie.stop`.
This takes no arguments and is useful with jumps as it allows for optional presentation parts.
```rhai
lie.stop()
```

To change the look and feel of the lie, use `lie.look`.
This takes a map which contains the values the user wishes to change.
```rhai
lie.look(#[ // Note: all fields are optional
    title: “foo”        // Set the title of the terminal window
    cwd: “/bar”,        // Set prompt directory
    user: "methos",     // Set prompt user
    host: "gaia",       // Set prompt host
    speed: 100,         // Set the typing speed
    final_prompt: true, // Set whether a final prompt is displayed
})
```

To _actually_ run commands and make changes on the underlying system, use `lie.system`, however, these are disabled by default.
To enable system calls, pass the `--unleash` flag and go play in dangerous mode.
The `lie.system` method has two forms.
The first takes a single string, pretends to type it then runs it on the host.
The second takes two strings, the first of which is the fake command to type and the second of which is the command to _actually_ run and which is hidden from the user.
```rhai
lie.system("adf");
lie.system("fake", "actual");
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

## Author, License and Name

This project is maintained by Ed Jones and is licensed under the GNU General Public License version 3.

The name ‘mendax’ is a Latin word which roughly translates to ‘narrator,’ ‘storyteller,’ or ‘habitual liar.’

[asciinema]: https://asciinema.org/
[rhai]: https://rhai.rs/book/
[snap]: https://snapcraft.io/mendax
