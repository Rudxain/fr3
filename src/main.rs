use clap::Parser;
use regex::bytes::Regex;
use std::{
	collections::HashMap,
	fs::{File, read_dir},
	io::{self, IsTerminal, Read, Write},
	path::{Path, PathBuf},
};

mod util;
#[allow(clippy::wildcard_imports, reason = "")]
use util::*;

fn f_counter(
	re: &Regex,
	counts: &mut HashMap<Box<[u8]>, usize>,
	mut f: File,
	buf: &mut Vec<u8>,
	err: &mut io::StderrLock,
) -> io::Result<()> {
	let cap = buf.capacity();
	// ensure mem is not dirty before mutation,
	// and ensure `reserve` is absolute rather than relative.
	buf.clear();
	match buf.try_reserve_exact(
		f.metadata()
			.map(|m| usize::try_from(m.len()).unwrap_or(usize::MAX))
			.unwrap_or(0),
	) {
		Ok(v) => v,
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
			debug_assert_eq!(cap, buf.capacity());
			// just-in-case the mem-pressure is extreme
			buf.shrink_to(cap.div_ceil(2));
			return Ok(());
		}
	}

	// NOTE: consider mem-maps as an alt to buffering.
	match f.read_to_end(buf) {
		Ok(_) => {
			counter(re.find_iter(buf).map(|m| m.as_bytes().into()), counts);
		}
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
		}
	}
	Ok(())
}

fn walker(
	re: &Regex,
	counts: &mut HashMap<Box<[u8]>, usize>,
	p: &Path,
	buf: &mut Vec<u8>,
	err: &mut io::StderrLock,
) -> io::Result<()> {
	// TO-DO: lock before checking if it's a dir
	if !p.is_dir() {
		return f_counter(
			re,
			counts,
			match File::open(p) {
				Ok(f) => f,
				Err(e) => {
					writeln!(err, "{e}")?;
					err.flush()?;
					return Ok(());
				}
			},
			buf,
			err,
		);
	}
	let dir = match read_dir(p) {
		Ok(d) => d,
		Err(e) => {
			writeln!(err, "{e}")?;
			err.flush()?;
			return Ok(());
		}
	};
	for entry in dir {
		let entry = match entry {
			Ok(de) => de,
			Err(e) => {
				writeln!(err, "{e}")?;
				err.flush()?;
				continue;
			}
		};
		let path = entry.path();
		// TO-DO: lock before checking if it's a dir
		if path.is_dir() {
			walker(re, counts, &path, buf, err)?;
		} else {
			// assume as regular file.
			// this may be wrong.
			match File::open(entry.path()) {
				Ok(f) => f_counter(re, counts, f, buf, err)?,
				Err(e) => {
					writeln!(err, "{e}")?;
					err.flush()?;
				}
			}
		}
	}
	Ok(())
}

/*
/// Matching type
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
	/// Find "words"
	#[default]
	Find,
	// Split "words" by delimiter
	Split,
}*/

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	/// Word regular-expression [default: \w{2,}]
	#[arg(short, long, value_name = "REGEX")]
	re: Option<Box<str>>,
	/// Sort words by count [default: true, if `stdout` is TTY]
	#[arg(short, long)]
	sort: Option<bool>,
	/// [default: .]
	paths: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	let mut out = io::stdout().lock();
	let mut err = io::stderr().lock();

	let re = match cli.re.as_deref() {
		Some(re) => match Regex::new(re) {
			Ok(re) => re,
			Err(e) => {
				writeln!(err, "{e}")?;
				err.flush()?;
				return Err(Box::new(e));
			}
		},
		// default
		// SAFETY: Clippy should've checked this
		_ => unsafe { Regex::new(r"\w{2,}").unwrap_unchecked() },
	};

	let mut p = cli.paths;
	if p.is_empty() {
		p.push(".".into());
	}
	let paths = p;

	// TTY check is also done by `clap` (`color` feature)
	// so this is redundant:
	// https://github.com/clap-rs/clap/discussions/6223
	let must_sort = cli.sort.unwrap_or_else(|| out.is_terminal());

	let mut buf: Vec<u8> = vec![];

	for path in paths {
		let mut counts = HashMap::new();
		walker(&re, &mut counts, path.as_ref(), &mut buf, &mut err)?;
		let mut counts: Box<[(_, _)]> = counts
			.into_iter()
			.map(|(k, c)| (String::from_utf8_lossy(&k).into_owned(), c))
			.collect();
		if must_sort {
			counts.sort_unstable_by_key(|&(_, c)| c);
			counts.reverse();
		}
		write!(out, "{}\n{}", path.display(), pretty_print_kv(counts))?;
		out.flush()?;
	}

	Ok(())
}
