Gitters
=======

Gitters is an implementation of the git tooling, written in Rust. I chose this project to understand how git works internally, as well as to learn Rust. The goal is to use gitters to manage the development of the project itself.

Quick start
-----------

Assuming you have Rust installed already:

```sh
git clone https://github.com/avik-das/gitters.git
cd gitters

# Run the tests. You'll need to ensure the "introduce-tests" branch is present
# in order for the tests to pass, as they check that a branch name resolves to
# the correct revision.
git fetch origin introduce-tests:introduce-tests
cargo test

# Very few git tools are available, but for the ones that are implemented,
# replace `git` with `./gitters` when executing the tool.

# Run "log" on a commit with a few ancestors:
./gitters log 44d6437947787a44b0e7d463954eef2daa44aaa5
./gitters log  # or just look at the history starting at HEAD

# See the configuration for this repository:
./gitters config --list
```

Supported commands
------------------

Use `./gitters <command> --help` to see the available options for each of the following commands.

- `branch`
- `cat-file`
- `config`
- `log`
- `ls-files`
- `rev-parse`
