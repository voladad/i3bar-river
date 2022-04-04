//! Some usefull functions

use serde::Deserialize;
use serde_json::{Deserializer, Error as JsonError};

/// Retuns (`last_line`, `remaining`). See tests for examples.
pub fn last_line(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut it = memchr::memrchr_iter(b'\n', s);
    let last = it.next()?;
    let rem = &s[(last + 1)..];
    if let Some(pre_last) = it.next() {
        Some((&s[(pre_last + 1)..last], rem))
    } else {
        Some((&s[..last], rem))
    }
}

/// Deserialize the last complete object. Returns (`object`, `remaining`). See tests for examples.
pub fn de_last_json<'a, T: Deserialize<'a>>(
    mut s: &'a [u8],
) -> Result<(Option<T>, &'a [u8]), JsonError> {
    let mut last = None;
    let mut tmp;
    loop {
        (tmp, s) = de_first_json(s)?;
        last = match tmp {
            Some(obj) => Some(obj),
            None => return Ok((last, s)),
        };
    }
}

/// Deserialize the first complete object. Returns (`object`, `remaining`). See tests for examples.
pub fn de_first_json<'a, T: Deserialize<'a>>(
    mut s: &'a [u8],
) -> Result<(Option<T>, &'a [u8]), JsonError> {
    while s
        .first()
        .map_or(false, |&x| x == b' ' || x == b',' || x == b'\n')
    {
        s = &s[1..];
    }
    let mut de = Deserializer::from_slice(s).into_iter();
    match de.next() {
        Some(Ok(obj)) => Ok((Some(obj), &s[de.byte_offset()..])),
        Some(Err(e)) if e.is_eof() => Ok((None, &s[de.byte_offset()..])),
        Some(Err(e)) => Err(e),
        None => Ok((None, &s[de.byte_offset()..])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! str {
        ($str:expr) => {
            &$str.as_bytes()[..]
        };
    }

    #[test]
    fn streaming_json() {
        let s = b",[2]\n, [3], [4, 3],[32][3] ";
        assert_eq!(
            de_first_json::<Vec<u8>>(s).unwrap(),
            (Some(vec![2]), str!("\n, [3], [4, 3],[32][3] "))
        );
        assert_eq!(
            de_last_json::<Vec<u8>>(s).unwrap(),
            (Some(vec![3]), str!(""))
        );

        let s = b",[2]\n, [3], [4, 3],[32][3] [2, 4";
        assert_eq!(
            de_last_json::<Vec<u8>>(s).unwrap(),
            (Some(vec![3]), str!("[2, 4"))
        );

        let s = b",[2]\n, [3], [4, 3],[32] invalid";
        assert_eq!(
            de_first_json::<Vec<u8>>(s).unwrap(),
            (Some(vec![2]), str!("\n, [3], [4, 3],[32] invalid"))
        );
        assert!(de_last_json::<Vec<u8>>(s).is_err());
    }

    #[test]
    fn test_last_line() {
        let s = b"hello";
        assert_eq!(last_line(s), None);

        let s = b"hello\n";
        assert_eq!(last_line(s), Some((str!("hello"), str!(""))));

        let s = b"hello\nworld";
        assert_eq!(last_line(s), Some((str!("hello"), str!("world"))));

        let s = b"hello\nworld\n";
        assert_eq!(last_line(s), Some((str!("world"), str!(""))));

        let s = b"hello\nworld\n...";
        assert_eq!(last_line(s), Some((str!("world"), str!("..."))));
    }
}
