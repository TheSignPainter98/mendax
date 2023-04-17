lie.run("echo foo", "foo");
lie.run("bar", ["fds", "fdas"]);
lie.run("baz", "r3");

lie.system("echo asdf");
lie.system("echo asdf", "ls");

lie.run("baz", "r3");

lie.look(#{
	fg: "red",
	bg: "black",
	title: "hfjdkla",
	cwd: "/",
	user: "methos",
	host: "gaia",
});

lie.cd("/bar/baz");

for cmd in ["foo", "bar", "baz"] {
	lie.screen("man " + cmd, |lie| {
		lie.show("hello, this is the manual page for " + cmd);
		lie.show("unfortunately this does not currently exist");
		lie.show("...");
		lie.show("sorry not sorry");
	})
}
