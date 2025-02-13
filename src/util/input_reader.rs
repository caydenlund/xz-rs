use std::io::BufRead;

pub trait InputRead: BufRead {
    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut bytes = [0u8; 1];
        self.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }

    fn read_be_u16(&mut self) -> std::io::Result<u16> {
        let mut bytes = [0u8; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }

    fn read_be_u32(&mut self) -> std::io::Result<u32> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }
}

impl<R: BufRead> InputRead for R {}
