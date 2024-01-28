use pyo3::exceptions;
use pyo3::prelude::*;

mod chunks;
use chunks::Chunker;
use fxhash::FxHashSet;

pub use alphabet_mask_models as models;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;

#[cfg(test)]
pub(crate) mod conftest;

/// Internal Rust function to mask a string.
fn mask_string(string: &str) -> Result<u32, String> {
    string.chars().try_fold(0_u32, |acc, c| {
        let char_code = c as u8;

        match char_code {
            32 => Ok(acc | 1),       // space
            46 => Ok(acc | 1 << 27), // full stop
            44 => Ok(acc | 1 << 28), // comma
            39 => Ok(acc | 1 << 29), // apostrophe
            45 => Ok(acc | 1 << 30), // hyphen
            34 => Ok(acc | 1 << 31), // double quote
            v if v & 64 == 0 || v & 128 != 0 => {
                Err(format!("String contains invalid character {c:?}."))
            }
            _ => Ok(acc | (1 << (char_code & 31))),
        }
    })
}

/// Convert a mask created from `mask_string` to a string of characters.
fn mask_to_chars(mask: u32) -> String {
    (0..=31_u8).fold(String::new(), |mut acc, i| {
        if mask & (1 << i) != 0 {
            match i {
                0 => acc.push(' '),
                27 => acc.push('.'),
                28 => acc.push(','),
                29 => acc.push('\''),
                30 => acc.push('-'),
                31 => acc.push('"'),
                _ => acc.push((i + 96) as char),
            }
        }
        acc
    })
}

/// Aggregate the results of a mask iterator by performing a bitwise AND on each result.
///
/// If any of the results are errors, the first error is returned.
fn intersect_masks<E>(mut masks: impl Iterator<Item = Result<u32, E>>) -> Result<u32, E> {
    masks.try_fold(u32::MAX, |acc, result| {
        if let Ok(mask) = result {
            Ok(acc & mask)
        } else {
            result
        }
    })
}

/// Returns a bit mask representing the common alphabet of the given strings.
fn find_common_mask<'s>(strings: impl Iterator<Item = &'s str>) -> Result<u32, String> {
    intersect_masks(strings.map(mask_string))
}

/// Chunk the given string iterator into chunks of at most `LENGTH_LIMIT_PER_CHUNK` bytes,
/// or a chunk of a single string if it is larger than `LENGTH_LIMIT_PER_CHUNK`.
fn chunk_strings_by<'s>(
    strings: impl Iterator<Item = &'s str>,
    length_limit: Option<usize>,
) -> impl Iterator<Item = Box<[&'s str]>> {
    if let Some(length_limit) = length_limit {
        Chunker::with_length_limit(strings, length_limit)
    } else {
        Chunker::new(strings)
    }
}

/// Returns a bit mask representing the common alphabet of the given strings,
/// using parallel processing.
fn find_common_mask_parallel<'s, T>(strings: T, length_limit: Option<usize>) -> Result<u32, String>
where
    T: ExactSizeIterator<Item = &'s str> + Send + Sync,
{
    let result = chunk_strings_by(strings, length_limit)
        .par_bridge()
        .map(
            // `into_vec()` should be fine here - there's no memcpy or allocation.
            |chunk| find_common_mask(chunk.into_vec().into_iter()),
        )
        .try_reduce(|| u32::MAX, |a, b| Ok(a & b));

    result
}

/// Returns a bit mask representing the alphabet of the given string.
///
/// Masked characters are:
/// - space (#0)
/// - A-Z (case insensitive) (#1-26)
/// - full stop (#27)
/// - comma (#28)
#[pyfunction]
fn alphabet_mask(string: &str, py: Python<'_>) -> PyResult<u32> {
    py.allow_threads(move || match mask_string(string) {
        Ok(mask) => Ok(mask),
        Err(e) => Err(exceptions::PyValueError::new_err(e)),
    })
}

/// Returns a bit mask representing the common alphabet of the given strings.
#[pyfunction]
fn common_alphabets(
    strings: Vec<&str>,
    length_limit: Option<usize>,
    py: Python<'_>,
) -> PyResult<String> {
    let length_limit = length_limit.unwrap_or(chunks::LENGTH_LIMIT_PER_CHUNK);

    let err_if_parallelise = strings.iter().try_fold(0_usize, |acc, s| {
        if let Some(new_len) = acc.checked_add(s.len()) {
            // Check for overflow
            if new_len > length_limit {
                return Err(()); // Use parallel processing
            }
            Ok(new_len) // We continue counting the total length
        } else {
            Err(()) // We've overflowed, so we should parallelise
        }
    });

    let strings = strings.into_iter();
    py.allow_threads(move || {
        macro_rules! expand_options {
                (
                    $($variant:ident => $func_call:expr),*$(,)?
                ) => {
                    match err_if_parallelise {
                        $(
                            $variant(_) => {
                                match $func_call {
                                    Ok(mask) => Ok(mask_to_chars(mask)),
                                    Err(e) => Err(exceptions::PyValueError::new_err(e))
                                }
                            }
                        )*
                    }
                };
            }

        expand_options!(
            Ok => find_common_mask(strings),
            Err => find_common_mask_parallel(strings, Some(length_limit))
        )
    })
}

/// Simply returns a set of the alphabet letters in the given string.
///
/// For speed comparisons only.
#[pyfunction]
fn alphabet_set(string: &str, py: Python<'_>) -> PyResult<PyObject> {
    let set = FxHashSet::from_iter(string.chars().map(|c| c.to_ascii_lowercase() as u8));
    Ok(set.into_py(py))
}

/// A Python module implemented in Rust.
#[pymodule]
fn lib_alphabet_mask(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(alphabet_mask, m)?)?;
    m.add_function(wrap_pyfunction!(alphabet_set, m)?)?;
    m.add_function(wrap_pyfunction!(common_alphabets, m)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! expand_tests {
        (
            $((
                $name:ident,
                $max:literal,
                $expected:expr
            ),)*$(,)?
        ) => {
            $(
                #[test]
                fn $name() {
                    let texts = conftest::COLLECTION_OF_50_CHARS_STRINGS[0..$max].to_vec();

                    let mask = find_common_mask_parallel(texts.into_iter(), Some(100)).unwrap();

                    assert_eq!(&mask_to_chars(mask), $expected);
                }
            )*
        };
    }

    expand_tests!(
        (test_1, 1, " acdeghilnprstw."),
        (test_2, 2, " adeghlnrstw."),
        (test_3, 3, " aehlrst."),
        (test_6, 6, " aehlrst."),
        (test_7, 7, " ehlrst."),
        (test_8, 8, " ehlrst."),
        (test_9, 9, " ehlrt."),
        (test_11, 11, " ehrt."),
        (test_15, 15, " ehrt."),
        (test_16, 16, " ert."),
        (test_19, 19, " ert."),
    );
}
