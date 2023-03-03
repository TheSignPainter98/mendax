lie.run("echo foo", "foo");
lie.run("bar", ["fds", "fdas"]);
lie.run("baz", "r3");

lie.system("echo asdf");
lie.system("ls", "echo asdf");

lie.prompt(#{
	user: "methos",
	host: "gaia",
	cwd: "/",
});

lie.run("baz", "r3");

lie.look(#{
	fg: "red",
	bg: "black",
	title: "hfjdkla",
});

lie.clear();

lie.system("ls", "echo asdf");
