# fr3
Performs [freq-analysis](https://en.wikipedia.org/wiki/Frequency_analysis) of ["words"](https://git-scm.com/docs/git-diff#Documentation/git-diff.txt---word-diff-regexregex), on directories and files.

Most of the code has been copy-pasted [from myself](https://github.com/Rudxain/loc.rs), lol

## Usage

### Install
This needs a Rust toolchain. Recommended command:
```sh
cargo install --path . --config 'build.rustflags="-C target-cpu=native"'
```
Assuming you've downloaded and `cd`ed into the repo

### Run
Invoke the program by passing the paths whose stats you want to get:
```sh
# example
fr3 file.txt directory/
file.txt
	test 3
	bruh 1
directory/
	the 64
	wiki 12
	yeet_3 6
# non-UTF8 paths are supported
```
Or simply pass nothing, if you want stats about WD, identical to `fr3 .`.

> [!important]
> The **default regex is for human use only**. It'll **always be API-unstable**, as it can change across patch versions and _maybe even across runs_ of the same version!

> [!note]
> [Tie-breaking](https://en.wikipedia.org/wiki/Tiebreaker) is unspecified.
> That is, if 2 or more words have the same count, they will be printed in arbitrary order within their [partition](https://doc.rust-lang.org/stable/std/primitive.slice.html#method.partition_point).
>
> I'm considering to lexicographically sort ties

You can define what a "word" is by passing [a regex](https://docs.rs/regex/1.12/regex/bytes/index.html):
```sh
fr3 --re '[^,\n\r]+' table.csv
table.csv
	my record 3
	joe 2
	zayda 1
	1 1
	0 1
	2 1
	id 1
	name 1
	column 1
```

Prepend [`(?i)` for case-insensitive regex](https://docs.rs/regex/1.12/regex/bytes/struct.RegexBuilder.html#method.case_insensitive). Note that **counting is always case-sensitive**:
```sh
fr3 --re '(?i)\bthe\b' prose.md
prose.md
	the 6
	The 3
	THE 1
```

The output format, while rudimentary, is mostly unambiguous. The only way (that I know) it can be ambiguous, is if your words contain `\n`, but that's just asking for trouble, so please let me pretend `\n` doesn't exist :3

Here's one last example usage; list your top 8 most used cmds:
```bash
fr3 -r.+ ~/.bash_history -strue | head -8
```
Note the `-strue` (`--sort=true`), this is needed because pipes aren't TTYs

## etc
This program is single-threaded, as it's IO-bound.

I'm considering to support matching delimiters by regex (`split`)

## See also
[`freq`](https://github.com/mre/freq)
