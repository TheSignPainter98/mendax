lie.run("echo foo", "foo");
lie.run("bar", ["fds", "fdas"]);
lie.run("baz", "r3");

lie.system("echo asdf");
lie.system("echo asdf", "echo FDSA<F4><F3><F2>");

lie.prompt(#{
	user: "methos",
	host: "gaia",
	cwd: "root",
});

lie.look(#{
	fg: "red",
	bg: "black",
});

lie.screen(|lie| {
	lie.run("man fdhsajkl", "as");
	lie
});
