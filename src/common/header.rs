use std::{io::Write, ptr::copy_nonoverlapping};

use crate::error::{new_io_err, Error};

pub const END: &[u8; 8] = b"END     ";

/// Implement iterator over empty keyword records to be written.
/// Internally use a buffer of 2.8 KB to write to the underlying `Write`
/// (thus **no need for a buffered write**, a buffered write will add a useless copy of the data).
pub struct HeaderWriter<'a, W: Write> {
  /// Destination
  writer: &'a mut W,
  /// Buffer overwritten in memory before being written in the `writer`
  /// (to minimize the number of calls of possibly faillible I/O operations.
  block2880: [u8; 2880],
  /// Current position in the block of 36 keyword records of 80 bytes each.
  position: u16,
}
impl<'a, W: Write> HeaderWriter<'a, W> {
  pub fn new(writer: &'a mut W) -> Self {
    Self {
      writer,
      block2880: [b' '; 2880],
      position: 0,
    }
  }

  fn flush(&mut self) -> Result<(), Error> {
    let res = self
      .writer
      .write_all(self.block2880.as_slice())
      .map_err(new_io_err);
    self.position = 0;
    self.block2880 = [b' '; 2880];
    res
  }

  /// Write the `END` keyword record, flush and returns.
  pub fn finalize(mut self) -> Result<(), Error> {
    // Copy 'END'
    if self.position == 2880 {
      self.flush()?;
    }
    let bytes = self.next_infallible();
    unsafe { copy_nonoverlapping(END.as_ptr(), bytes.as_mut_ptr(), END.len()) };
    // Write the current (last) chunk and return
    // (do not use self.flush() to avoid the allocation of 2880 bytes)
    self
      .writer
      .write_all(self.block2880.as_slice())
      .map_err(new_io_err)
  }

  /// # WARNING
  /// Internal use only, **be sure to have test position < 2880 before**!!
  fn next_infallible(&mut self) -> &'a mut [u8; 80] {
    let new_pos = self.position + 80;
    let kwr: &mut [u8] = &mut self.block2880[self.position as usize..new_pos as usize];
    self.position = new_pos;
    unsafe { &mut *(kwr.as_mut_ptr().cast::<[u8; 80]>()) }
  }
}

impl<'a, W: Write> Iterator for HeaderWriter<'a, W> {
  type Item = Result<&'a mut [u8; 80], Error>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.position < 2880 {
      Some(Ok(self.next_infallible()))
    } else {
      Some(self.flush().map(|()| self.next_infallible()))
    }
  }
}
