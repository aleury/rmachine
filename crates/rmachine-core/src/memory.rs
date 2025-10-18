#![allow(clippy::unreadable_literal)]
use anyhow::Result;
use anyhow::anyhow;

pub type Address = usize;

/// Byte-addressable memory.
#[derive(Debug, Default, PartialEq)]
pub struct Memory(pub(crate) Vec<u8>);

impl Memory {
    #[must_use]
    pub fn new(memory_size: usize) -> Self {
        Self(vec![0u8; memory_size])
    }

    /// Load bytes into memory at the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn load(&mut self, addr: Address, bytes: &[u8]) -> Result<()> {
        let slice = self
            .0
            .get_mut(addr..addr + bytes.len())
            .ok_or_else(|| anyhow!("memory out of bounds: {addr:#x}"))?;
        slice.copy_from_slice(bytes);
        Ok(())
    }

    /// Read a byte from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u8(&self, addr: Address) -> Result<u8> {
        self.0
            .get(addr)
            .copied()
            .ok_or_else(|| anyhow!("memory out of bounds: {addr:#x}"))
    }

    /// Write a byte to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u8(&mut self, addr: Address, value: u8) -> Result<()> {
        let byte = self
            .0
            .get_mut(addr)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        *byte = value;
        Ok(())
    }

    /// Read a 16-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u16(&self, addr: Address) -> Result<u16> {
        let bytes: [u8; 2] = self
            .0
            .get(addr..addr + 2)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u16::from_le_bytes(bytes))
    }

    /// Write a 16-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u16(&mut self, addr: Address, value: u16) -> Result<()> {
        let bytes = self
            .0
            .get_mut(addr..addr + 2)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Read a 32-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u32(&self, addr: Address) -> Result<u32> {
        let bytes: [u8; 4] = self
            .0
            .get(addr..addr + 4)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Write a 32-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u32(&mut self, addr: Address, value: u32) -> Result<()> {
        let bytes = self
            .0
            .get_mut(addr..addr + 4)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Read a 64-bit value from memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn read_u64(&self, addr: Address) -> Result<u64> {
        let bytes: [u8; 8] = self
            .0
            .get(addr..addr + 8)
            .ok_or_else(|| anyhow!("memory read out of bounds: {addr:#x}"))?
            .try_into()?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// Write a 64-bit value to memory.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn write_u64(&mut self, addr: Address, value: u64) -> Result<()> {
        let bytes = self
            .0
            .get_mut(addr..addr + 8)
            .ok_or_else(|| anyhow!("memory write out of bounds: {addr:#x}"))?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_loads_bytes_at_the_given_address() {
        let mut memory = Memory::new(6);
        memory.load(2, &[0x01, 42]).unwrap();

        let want = vec![0, 0, 1, 42, 0, 0];
        let got = memory.0;
        assert_eq!(want, got, "bytes should be loaded into memory");
    }

    #[test]
    fn read_u8_fn_reads_byte_from_memory_at_given_address() {
        let memory = Memory(vec![42]);

        let want = 42;
        let got = memory.read_u8(0).unwrap();
        assert_eq!(want, got, "byte should be read from memory");
    }

    #[test]
    fn write_u8_fn_writes_byte_to_memory_at_given_address() {
        let mut state = Memory::new(1);

        state
            .write_u8(0, 42)
            .expect("Failed to write byte to memory");

        let want = 42;
        let got = state.read_u8(0).unwrap();
        assert_eq!(
            want, got,
            "byte should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u16_fn_reads_u16_from_memory_at_given_address() {
        let memory = Memory(vec![0xcd, 0xab]);

        let want = 0xabcd;
        let got = memory.read_u16(0).unwrap();
        assert_eq!(want, got, "u16 should be read from memory");
    }

    #[test]
    fn write_u16_fn_writes_u16_to_memory_at_given_address() {
        let mut memory = Memory::new(2);

        memory
            .write_u16(0, 0xabcd)
            .expect("Failed to write u16 to memory");

        let want = vec![0xcd, 0xab];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u16 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u32_fn_reads_u32_from_memory_at_given_address() {
        let memory = Memory(vec![0xef, 0xbe, 0xad, 0xde]);

        let want = 0xdeadbeef;
        let got = memory.read_u32(0).unwrap();
        assert_eq!(want, got, "u32 should be read from memory in little-endian");
    }

    #[test]
    fn write_u32_fn_writes_u32_to_memory_at_given_address() {
        let mut memory = Memory::new(4);

        memory
            .write_u32(0, 0xdeadbeef)
            .expect("Failed to write u32 to memory");

        let want = vec![0xef, 0xbe, 0xad, 0xde];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u32 should be written to memory in little-endian"
        );
    }

    #[test]
    fn read_u64_fn_reads_u64_from_memory_at_given_address() {
        let memory = Memory(vec![0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde]);

        let want = 0xdeadbeefcafebabe;
        let got = memory.read_u64(0).unwrap();
        assert_eq!(want, got, "u64 should be read from memory in little-endian");
    }

    #[test]
    fn write_u64_fn_writes_u64_to_memory_at_given_address() {
        let mut memory = Memory::new(8);

        memory
            .write_u64(0, 0xdeadbeefcafebabe)
            .expect("Failed to write u64 to memory");

        let want = vec![0xbe, 0xba, 0xfe, 0xca, 0xef, 0xbe, 0xad, 0xde];
        let got = memory.0;
        assert_eq!(
            want, got,
            "u64 should be written to memory in little-endian"
        );
    }
}
