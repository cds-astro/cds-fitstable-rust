use std::{convert::TryInto, io::Read};
// RENAME chunk2880 by block??

pub mod common;
pub mod error;
pub mod hdu;

pub mod read;

/*
fn parse(card: &[u8]) {
  let card = b"TFORM666";
  match card {
    b"TFORM777" => println!("FTORM777 found!"),
    [b'T', b'F', b'O', b'R', b'M', num @ ..] => println!("FTORM: {}", str::from_utf8(num).unwrap()),
    b"TFORM666" => println!("FTORM666 found!"),
    _ => println!("Toto: {}", str::from_utf8(card).unwrap()),
  }
}
*/

/*
pub struct RawHeader {
    chunks: Vec<[u8; 2880]>,
    end_position: usize,
}
impl RawHeader {
    /// Read the Header from the given reader.
    /// * The reader position must be the first byte of a header.
    /// * The total number of bytes read is exactly the `byte_size` of the header.
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        // Method detecting the position of "END" keyword, if any.
        fn end_position(chunk2880: &[u8; 2880]) -> Option<usize> {
            debug_assert_eq!(chunk2880.len(), 2880);
            for (i, chunk) in chunk2880.chunks(80).enumerate() {
                if chunk.starts_with(b"END     ") {
                    return Some(i);
                }
            }
            None
        }
        // Read the header by chunks of 2880
        let mut chunks = Vec::with_capacity(6);
        loop {
            let mut chunk2880 = [0_u8; 2880];
            reader.read_exact(&mut chunk2880)?;
            let end_position = end_position(&chunk2880);
            chunks.push(chunk2880);
            if let Some(mut end_position) = end_position {
                end_position += 36 * (chunks.len() - 1);
                return Ok(Self { chunks, end_position })
            }
        }
    }

    /// Returns the size, in bytes, of the header (it is a multiple of 2880).
    pub fn byte_size(&self) -> usize {
        2880 * self.chunks.len()
    }

    /// Iterates on the header, 80 bytes by 80 bytes.
    /// The iterator stops before the `END` card. So the `END` card is **NOT** returned.
    pub fn chunks80(&self) -> impl Iterator<Item = &[u8]>{
        self.chunks.iter().flat_map(|chunk| chunk.chunks(80)).take(self.end_position)
    }

    /// Iterates on the header blocks, 2880 bytes by 2880 bytes.
    /// # TIP
    /// Made to re-write the header if needed.
    pub fn chunks2880(&self) -> impl Iterator<Item = &[u8]>{
        self.chunks.iter().map(|s| s.as_slice())
    }
}

impl<'a> IntoIterator for &'a RawHeader {
    type Item = &'a [u8];
    type IntoIter = Take<FlatMap<Iter<'a, [u8; 2880]>, Chunks<'a, u8>, fn(&[u8; 2880]) -> Chunks<u8>>>;

    fn into_iter(self) -> Self::IntoIter {
        let f: fn(&[u8; 2880]) -> Chunks<u8> = |chunk: &[u8; 2880]| chunk.chunks(80);
        self.chunks.iter().flat_map(f).take(self.end_position)
    }
}
*/

/*
pub fn add(left: u64, right: u64) -> u64 {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
*/
