use crate::err::{CursorError, ParseError};
pub struct Cursor {
    data: Vec<u8>,
    pos: usize,
}
pub struct f2_14(i16);
impl f2_14 {
    pub fn to_f32(self) -> f32 {
        self.0 as f32 / 16384.0
    }
    pub fn from_f32(val: f32) -> Self {
        f2_14((val * 16384.0).round() as i16)
    }
}

impl Cursor {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            data: buffer,
            pos: 0,
        }
    }
    #[inline(always)]
    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], ParseError> {
        let bytes = self
            .data
            .get(self.pos..self.pos + N)
            .ok_or(ParseError::UnexpectedEof)?;
        self.pos += N;
        Ok(bytes.try_into().unwrap())
    }
    #[inline(always)]
    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        Ok(u8::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u16(&mut self) -> Result<u16, ParseError> {
        Ok(u16::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u32(&mut self) -> Result<u32, ParseError> {
        Ok(u32::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u64(&mut self) -> Result<u64, ParseError> {
        Ok(u64::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i16(&mut self) -> Result<i16, ParseError> {
        Ok(i16::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i32(&mut self) -> Result<i32, ParseError> {
        Ok(i32::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i64(&mut self) -> Result<i64, ParseError> {
        Ok(i64::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_f32(&mut self) -> Result<f32, ParseError> {
        Ok(f32::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_f2dot14(&mut self) -> Result<f2_14, ParseError> {
        Ok(f2_14(i16::from_be_bytes(self.read_bytes()?)))
    }
    #[inline(always)]
    pub fn seek(&mut self, pos: usize) -> Result<(), ParseError> {
        if pos > self.data.len() {
            return Err(ParseError::OutOfBounds {
                pos,
                len: self.data.len(),
            });
        }
        self.pos = pos;
        Ok(())
    }
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }
}
