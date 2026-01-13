use clap::Parser;
use regex::bytes::Regex;
use std::{
	collections::HashMap,
	fs::File,
	io::{self, IsTerminal, Read, Write},
	path::PathBuf,
};
use walkdir::WalkDir;

mod util;
#[allow(clippy::wildcard_imports, reason = "")]
use util::*;

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
		let mut counts: HashMap<Box<[u8]>, usize> = HashMap::new();
		for rde in WalkDir::new(&path)
			.max_open(8) // power of 2 that's closest to `10` (last known default)
			.follow_links(true)
		// not using `filter_map`, to avoid `stderr` lifetime issues
		{
			let de = match rde {
				Ok(de) => de,
				Err(e) => {
					write!(&mut err, "{e}")?;
					err.flush()?;
					continue;
				}
			};
			let mut f = if de.file_type().is_file() {
				match File::open(de.path()) {
					Ok(f) => f,
					Err(e) => {
						write!(&mut err, "{e}")?;
						err.flush()?;
						continue;
					}
				}
			} else {
				continue;
			};

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
			match f.read_to_end(&mut buf) {
				Ok(_) => {
					counter(re.find_iter(&buf).map(|m| m.as_bytes().into()), &mut counts);
				}
				Err(e) => {
					writeln!(err, "{e}")?;
					err.flush()?;
				}
			}
		}
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
