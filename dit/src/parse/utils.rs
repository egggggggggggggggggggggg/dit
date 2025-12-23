use crate::parse::ReadError;
pub struct Cursor<'a> {
    data: &'a [u8],
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

impl<'a> Cursor<'a> {
    #[inline(always)]
    pub fn set(data: &'a [u8], pos: usize) -> Self {
        Self { data, pos }
    }
    #[inline(always)]
    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
        let bytes = self
            .data
            .get(self.pos..self.pos + N)
            .ok_or(ReadError::OutOfBounds)?;
        self.pos += N;
        Ok(bytes.try_into().unwrap())
    }
    #[inline(always)]
    pub fn read_u8(&mut self) -> Result<u8, ReadError> {
        Ok(u8::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u16(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u32(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_be_bytes(self.read_bytes()?))
    }
    #[inline(always)]
    pub fn read_u64(&mut self) -> Result<u64, ReadError> {
        Ok(u64::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i16(&mut self) -> Result<i16, ReadError> {
        Ok(i16::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i32(&mut self) -> Result<i32, ReadError> {
        Ok(i32::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_i64(&mut self) -> Result<i64, ReadError> {
        Ok(i64::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_f32(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_be_bytes(self.read_bytes()?))
    }

    #[inline(always)]
    pub fn read_f2dot14(&mut self) -> Result<f2_14, ReadError> {
        Ok(f2_14(i16::from_be_bytes(self.read_bytes()?)))
    }
    #[inline(always)]
    pub fn seek(&mut self, pos: usize) -> Result<(), ReadError> {
        if pos > self.data.len() {
            return Err(ReadError::OutOfBounds);
        }
        self.pos = pos;
        Ok(())
    }
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.pos
    }
}
