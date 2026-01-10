use std::{collections::HashMap, fmt::Display, fmt::Write, hash::Hash};

/// `ExactSizeIterator` is necessary
/// to guarantee the counters won't overflow
pub fn counter<I, T>(it: I, counts: &mut HashMap<T, usize>)
where
	I: IntoIterator<Item = T>,
	//I::IntoIter: ExactSizeIterator,
	T: Eq + Hash,
{
	for x in it {
		counts
			.entry(x)
			// why `checked_add` causes inference to totally fail?
			.and_modify(|c: &mut usize| *c = (*c).checked_add(1).unwrap_or_else(|| unreachable!()))
			.or_insert(1);
	}
}

pub fn pretty_print_kv<K: Display, V: Display, I: IntoIterator<Item = (K, V)>>(map: I) -> String {
	let mut s = String::new();
	for (k, v) in map {
		let _ = writeln!(s, "\t{k} {v}");
	}
	s
}
