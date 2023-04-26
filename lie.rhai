lie.run("echo foo", "foo\n");
lie.run("bar", ["fds", "fdas\n"]);
lie.run("baz", "r3\n");
lie.run("asd");
lie.run("fdsa");

lie.tag("hello");
lie.system("echo asdf");
lie.system("echo asdf", "ls");

lie.tag("world");
lie.run("baz", "r3\n");

lie.look(#{
	title: "hfjdkla",
	cwd: "/",
	user: "methos",
	host: "gaia",
});

lie.cd("/bar/baz");

for cmd in ["foo", "bar", "baz"] {
	lie.screen("man " + cmd, |lie| {
		let lines = [
			`hello, this is the manual page for ${cmd}`,
			"unfortunately this does not currently exist",
			"...",
			"sorry not sorry",
		];
		for line in lines {
			lie.show(line + "\n");
		}
	})
}
