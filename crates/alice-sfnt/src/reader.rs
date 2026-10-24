//! Big-endian reads over a byte slice. Every read and every declared array is checked against the bytes left
//! first (sizes are computed in u64, so no count can wrap); nothing here can panic.

use crate::error::{truncated, Result};

/// `data[offset..offset + len]`, or an error when it does not fit.
pub(crate) fn slice<'d>(
    data: &'d [u8],
    offset: u64,
    len: u64,
    context: &'static str,
) -> Result<&'d [u8]> {
    let end = offset.checked_add(len).ok_or(truncated(context))?;
    if end > data.len() as u64 {
        return Err(truncated(context));
    }
    // both bounds are <= data.len(), which is a usize
    Ok(&data[offset as usize..end as usize])
}

/// `data[offset..]`, or an error when the offset is past the end.
pub(crate) fn tail<'d>(data: &'d [u8], offset: u64, context: &'static str) -> Result<&'d [u8]> {
    if offset > data.len() as u64 {
        return Err(truncated(context));
    }
    Ok(&data[offset as usize..])
}

/// A cursor over a byte slice.
#[derive(Clone, Copy)]
pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    context: &'static str,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8], context: &'static str) -> Self {
        Reader {
            data,
            pos: 0,
            context,
        }
    }

    pub(crate) fn at(data: &'a [u8], offset: u64, context: &'static str) -> Result<Self> {
        Ok(Reader {
            data: tail(data, offset, context)?,
            pos: 0,
            context,
        })
    }

    pub(crate) fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub(crate) fn at_end(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub(crate) fn bytes(&mut self, len: u64) -> Result<&'a [u8]> {
        if len > self.remaining() as u64 {
            return Err(truncated(self.context));
        }
        let s = &self.data[self.pos..self.pos + len as usize];
        self.pos += len as usize;
        Ok(s)
    }

    /// `count` records of `size` bytes each, checked as a whole before anything is read.
    pub(crate) fn array(&mut self, count: u64, size: u64) -> Result<&'a [u8]> {
        let len = count.checked_mul(size).ok_or(truncated(self.context))?;
        self.bytes(len)
    }

    pub(crate) fn skip(&mut self, len: u64) -> Result<()> {
        self.bytes(len).map(|_| ())
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N]> {
        let b = self.bytes(N as u64)?;
        let mut out = [0u8; N];
        out.copy_from_slice(b);
        Ok(out)
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.fixed::<1>()?[0])
    }

    pub(crate) fn i8(&mut self) -> Result<i8> {
        Ok(self.u8()? as i8)
    }

    pub(crate) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.fixed()?))
    }

    pub(crate) fn i16(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.fixed()?))
    }

    pub(crate) fn u24(&mut self) -> Result<u32> {
        let b: [u8; 3] = self.fixed()?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.fixed()?))
    }

    pub(crate) fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    /// an unsigned integer of 1..=4 bytes (CFF offsets)
    pub(crate) fn uint(&mut self, size: u8) -> Result<u32> {
        let b = self.bytes(u64::from(size))?;
        Ok(b.iter().fold(0u32, |acc, &x| (acc << 8) | u32::from(x)))
    }
}

/// big-endian u16 at `index` of an array of u16 (the array was bounds-checked when it was taken)
pub(crate) fn u16_at(array: &[u8], index: usize, context: &'static str) -> Result<u16> {
    let at = index.checked_mul(2).ok_or(truncated(context))?;
    let b = array.get(at..at + 2).ok_or(truncated(context))?;
    Ok(u16::from_be_bytes([b[0], b[1]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorKind;

    #[test]
    fn a_count_past_the_end_is_an_error_before_any_read() {
        let data = [0u8; 8];
        let mut r = Reader::new(&data, "t");
        assert_eq!(
            r.array(u64::from(u32::MAX), 2).unwrap_err().kind,
            ErrorKind::Truncated
        );
        assert_eq!(
            r.array(u64::MAX, u64::MAX).unwrap_err().kind,
            ErrorKind::Truncated
        );
        assert_eq!(r.remaining(), 8);
        assert_eq!(r.array(4, 2).unwrap().len(), 8);
        assert!(r.u8().is_err());
    }

    #[test]
    fn slices_do_not_wrap() {
        let data = [1u8, 2, 3];
        assert!(slice(&data, u64::MAX, 2, "t").is_err());
        assert!(slice(&data, 2, 2, "t").is_err());
        assert_eq!(slice(&data, 1, 2, "t").unwrap(), &[2, 3]);
        assert!(tail(&data, 4, "t").is_err());
        assert_eq!(tail(&data, 3, "t").unwrap(), &[] as &[u8]);
    }

    #[test]
    fn integers_are_big_endian() {
        let data = [0x12, 0x34, 0x56, 0x78, 0xFF, 0xFE];
        let mut r = Reader::new(&data, "t");
        assert_eq!(r.u24().unwrap(), 0x123456);
        assert_eq!(r.u8().unwrap(), 0x78);
        assert_eq!(r.i16().unwrap(), -2);
        let mut r = Reader::new(&data, "t");
        assert_eq!(r.uint(3).unwrap(), 0x123456);
        assert_eq!(u16_at(&data, 2, "t").unwrap(), 0xFFFE);
        assert!(u16_at(&data, 3, "t").is_err());
    }
}
