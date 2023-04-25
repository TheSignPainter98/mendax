# Mendax

A CLI spoofer—like [asciinema][asciinema] but in real-time.
Specify inputs to fake and outputs to fake, all of which may be checked before running.

## Table of Contents

<!-- vim-markdown-toc GFM -->

* [Why?](#why)
* [Installation](#installation)
* [Not sure where to start?](#not-sure-where-to-start)
* [Writing the lie](#writing-the-lie)
* [Author, License and Name](#author-license-and-name)

<!-- vim-markdown-toc -->

## Why?

In my experience, a demo of a product is _far_ more likely to break than production code.
If code is in production, it’s likely gone through many layers of testing and checking so there’s a pretty good chance it does what it says on the tin.
If code is used in demos, suddenly, it’s a completly different matter.
Perhaps that minor change you made just fixing ‘that one last thing’ before the presentation turns out to break something critical.
Perhaps your internet connection decides it just doesn’t want to work anymore, despite being fine ten minutes earlier.
Suddenly, due factors which may be beyond your control, you’re now awkwardly filling time trying to fix code, questioning both the program itself and the life choices which led up to this moment.

Well no more.

Mendax offers:

- **Sandboxed spoofing**—unless you specifically allow it, mendax runs in a complete sandbox, unable to read or alter the host system’s state. No more surprises!
- **Dry running**—demos can be checked ahead of time. Higher quality in less development time.
- **Time travel**—tag important moments in the demo so you can jump between them at showtime. Adapt to your audience.

Oh and one more thing, do you creating [asciinema][asciinema] recordings and getting annoyed at your typing speed and spelling mistakes?
Well I do, and quite a bit; what should be a five minute job can stretch for far longer as I stress about the tiniest details.
All this can go away just by running—
```bash
asciinema rec -c mendax
```

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

## Author, License and Name

This project is maintained by Ed Jones and is licensed under the GNU General Public License version 3.

The name ‘mendax’ is a Latin word which roughly translates to ‘narrator,’ ‘storyteller,’ or ‘habitual liar.’
Exactly which meaning is the most appropriate is left as an exercise for the reader.

[asciinema]: https://asciinema.org/
[rhai]: https://rhai.rs/book/
[snap]: https://snapcraft.io/mendax
