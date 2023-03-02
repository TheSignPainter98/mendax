lie.fake("echo foo", "foo");
lie.fake("bar", "fds");
lie.fake("baz", "r3");

lie.run("echo fda");

lie.prompt(#{
	user: "methos",
	host: "gaia",
	cwd: "root",
});

lie.look(#{
	fg: "red",
	bg: "black",
});

fn foo(lie) {
	lie.run("man fdhsajkl");
	lie
}
lie.screen(foo);
lie.screen(Fn("foo"));
lie.screen(|lie| {
	lie.run("man fdhsajkl");
});
