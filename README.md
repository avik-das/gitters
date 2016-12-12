Gitters
=======

Gitters is an implementation of the git tooling, written in Rust. I chose this project to understand how git works internally, as well as to learn Rust. The goal is to use gitters to manage the development of the project itself.

Quick Start
-----------

Assuming you have Rust installed already:

```sh
git clone https://github.com/avik-das/gitters.git
cd gitters

# Very few git tools are available, but for the ones that are implemented,
# replace `git` with `./gitters` when executing the tool.

# Run "cat-file" on the initial commit:
./gitters cat-file -p 4ddb0025ef5914b51fb835495f5259a6d962df21

# Run "log" on a commit with a few ancestors:
./gitters log 44d6437947787a44b0e7d463954eef2daa44aaa5

# See the configuration for this repository:
./gitters config --list
```
