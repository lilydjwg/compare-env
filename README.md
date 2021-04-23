Compare the given environment variable value across all the processes.

To install, install Rust first then run:

```
cargo install --path .
```

You'll get a binary called `compare-env` in `~/.cargo/bin` (you should add this to your `$PATH`).

Sample usage output:

```
$ compare-env LANG
    2 Nothing ([10081, 26474])
   29 Value("en_US.UTF-8") ([694, 766, .......])
   41 Value("zh_CN.UTF-8") ([1005, 1122, .......])
  512 Fail ([1, 2, 3, 5, .......])
```
