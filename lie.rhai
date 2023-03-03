lie.run("echo foo", "foo");
lie.run("bar", ["fds", "fdas"]);
lie.run("baz", "r3");

lie.system("echo asdf");
lie.system("ls", "echo asdf");

lie.run("baz", "r3");

lie.look(#{
	fg: "red",
	bg: "black",
	title: "hfjdkla",
	user: "methos",
	host: "gaia",
	cwd: "/",
});

lie.clear();

lie.system("ls", "super-cool-command-not-ls");
