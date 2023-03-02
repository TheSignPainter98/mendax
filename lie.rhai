lie.fake("echo foo", "foo");
lie.fake("bar", "fds");
lie.fake("baz", "r3");

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
	lie.fake("man fdhsajkl", "as");
	lie
});
