//! A chunker struct to chunk strings into chunks of at most `LENGTH_LIMIT_PER_CHUNK` bytes,
//! or a chunk of a single string if it is larger than `LENGTH_LIMIT_PER_CHUNK`.
//!
use std::iter::Peekable;

/// The maximum number of bytes to process in a single chunk.
pub const LENGTH_LIMIT_PER_CHUNK: usize = 1 << 20; // 1 MiB

/// A chunker struct to chunk strings into chunks of a maximum length, or a
/// chunk of a single string if it is larger than the maximum length.
pub(crate) struct Chunker<'s, T>
where
    T: Iterator<Item = &'s str>,
{
    strings: Peekable<T>,
    length_limit: usize,
}

impl<'s, T> Chunker<'s, T>
where
    T: Iterator<Item = &'s str>,
{
    /// Create a new chunker.
    pub(crate) fn new(strings: T) -> Self {
        Self::with_length_limit(strings, LENGTH_LIMIT_PER_CHUNK)
    }

    /// Create a new chunker with the given length limit.
    pub(crate) fn with_length_limit(strings: T, length_limit: usize) -> Self {
        Self {
            strings: strings.peekable(),
            length_limit,
        }
    }
}

impl<'s, T> Iterator for Chunker<'s, T>
where
    T: Iterator<Item = &'s str>,
{
    type Item = Box<[&'s str]>;

    fn next(&mut self) -> Option<Box<[&'s str]>> {
        let mut chunk = Vec::new();
        let mut length = 0;

        while let Some(string) = self.strings.peek() {
            let string_length = string.len();
            if length + string_length > self.length_limit {
                if chunk.is_empty() {
                    // This is safe because we already peeked.
                    let string = self.strings.next().unwrap();

                    // The string is too long to fit in a single chunk.
                    // Return a chunk of a single string.
                    return Some(vec![string].into_boxed_slice());
                } else {
                    // The string is too long to fit in the current chunk.
                    // Return the current chunk.
                    return Some(chunk.into_boxed_slice());
                }
            } else {
                // This is safe because we already peeked.
                let string = self.strings.next().unwrap();

                // The string fits in the current chunk.
                chunk.push(string);
                length += string_length;
            }
        }

        if chunk.is_empty() {
            None
        } else {
            // Return the remaining chunk, whatever its length.
            Some(chunk.into_boxed_slice())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::conftest;

    /// Test the chunker.
    #[test]
    fn simple() {
        let strings = vec![
            "Hello, world!",                                                      // len = 13
            "The quick brown fox jumps over the lazy dog.",                       // len = 43
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",           // len = 57
            "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.", // len = 63
            "Ending",                                                             // len = 6
        ];
        // 57 is chosen because the first two strings add up to 56 bytes.
        let chunker = Chunker::with_length_limit(strings.into_iter(), 57);
        let chunks: Vec<_> = chunker.collect();

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].len(), 2);
        assert_eq!(
            chunks[0],
            vec![
                "Hello, world!",
                "The quick brown fox jumps over the lazy dog."
            ]
            .into_boxed_slice()
        );
        assert_eq!(chunks[1].len(), 1);
        assert_eq!(
            chunks[1],
            vec!["Lorem ipsum dolor sit amet, consectetur adipiscing elit."].into_boxed_slice()
        );
        assert_eq!(chunks[2].len(), 1);
        assert_eq!(
            chunks[2],
            vec!["Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."]
                .into_boxed_slice()
        );
        assert_eq!(chunks[3].len(), 1);
        assert_eq!(chunks[3], vec!["Ending"].into_boxed_slice());
    }

    #[test]
    fn empty() {
        let strings = Vec::<&str>::new();
        let chunker = Chunker::new(strings.into_iter());
        let chunks: Vec<_> = chunker.collect();

        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn uniform_strings_100() {
        let strings = crate::conftest::COLLECTION_OF_50_CHARS_STRINGS;

        const LENGTH_LIMIT: usize = 100;

        let chunker = Chunker::with_length_limit(strings.into_iter(), LENGTH_LIMIT);

        for (i, chunk) in chunker.enumerate() {
            assert!(chunk.len() == 2);
            assert!(chunk[0] == conftest::COLLECTION_OF_50_CHARS_STRINGS[i * 2]);
            assert!(chunk[1] == conftest::COLLECTION_OF_50_CHARS_STRINGS[i * 2 + 1]);

            assert!(chunk.iter().map(|s| s.len()).sum::<usize>() <= LENGTH_LIMIT);
        }
    }
}
