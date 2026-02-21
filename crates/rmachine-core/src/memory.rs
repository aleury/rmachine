use anyhow::{Context, Result, anyhow, bail};
#[must_use]
pub struct Memory(Vec<u8>);

impl Memory {
    pub fn new(size: usize) -> Self {
        Self(vec![0; size])
    }

    /// Load bytes into memory at the given address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds.
    pub fn load(&mut self, addr: u16, program: &[u8]) -> Result<()> {
        let start = usize::from(addr);
        let end = start
            .checked_add(program.len())
            .context("program too big")?
            .checked_sub(1)
            .context("program empty")?;
        let max = self.0.len().checked_sub(1).context("zero memory")?;
        if end > max {
            bail!("end address beyond memory ({end:#X} against {max:#X})")
        }
        let slice = self
            .0
            .get_mut(start..=end)
            .ok_or_else(|| anyhow!("invalid memory range: {start:#X}-{end:#X} (max {max:#X}"))?;
        slice.copy_from_slice(program);
        Ok(())
    }

    /// Returns the byte at the given address.
    #[must_use]
    pub fn get8(&self, addr: u16) -> u8 {
        self.0.get(usize::from(addr)).copied().unwrap_or(0)
    }

    /// Returns the (little-endian) word at the given address.
    ///
    /// No alignment requirements apply.
    #[must_use]
    pub fn get16(&self, addr: u16) -> u16 {
        let le_bytes = [self.get8(addr), self.get8(addr.wrapping_add(1))];
        u16::from_le_bytes(le_bytes)
    }

    /// Writes a byte to memory at the given address.
    pub fn set8(&mut self, addr: u16, value: u8) {
        if let Some(byte) = self.0.get_mut(usize::from(addr)) {
            *byte = value;
        }
    }

    /// Writes a (little-endian) word to memory at the given address.
    ///
    /// No alignment requirements apply.
    pub fn set16(&mut self, addr: u16, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.set8(addr, lo);
        self.set8(addr.wrapping_add(1), hi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get8_returns_a_byte_from_memory() {
        let mut machine = Memory::new(1024);
        machine.load(0, &[0xFF]).unwrap();

        assert_eq!(machine.get8(0), 0xFF);
    }

    #[test]
    fn get16_returns_le_word_from_memory() {
        let mut machine = Memory::new(1024);

        machine.load(0, &[0xEF, 0xBE]).unwrap();

        assert_eq!(machine.get16(0), 0xBEEF);
    }

    #[test]
    fn get16_returns_le_word_from_memory_at_wrapping_addr() {
        let mut m = Memory::new(0x10_000);
        m.set8(0xFFFF, 0xEF);
        m.set8(0x0000, 0xBE);
        assert_eq!(m.get16(0xFFFF), 0xBEEF, "wrong value");
    }

    #[test]
    fn set8_writes_a_byte_to_memory() {
        let mut m = Memory::new(1024);

        m.set8(0, 0x42);

        assert_eq!(m.get8(0), 0x42);
    }

    #[test]
    fn set8_ignores_out_of_bounds_writes() {
        let mut m = Memory::new(1024);

        // Memory size is 1024 bytes (0-1023), so address 2000 is out of bounds
        m.set8(2000, 0xFF);

        // Should not panic, and reading out of bounds returns 0
        assert_eq!(m.get8(2000), 0x00);
    }

    #[test]
    fn set16_writes_le_word_to_memory() {
        let mut m = Memory::new(1024);
        m.set16(0, 0xBEEF);
        assert_eq!(m.get8(0), 0xEF, "wrong low byte");
        assert_eq!(m.get8(1), 0xBE, "wrong high byte");
    }

    #[test]
    fn set16_writes_le_word_to_memory_at_wrapping_addr() {
        let mut m = Memory::new(0x10_000);
        m.set16(0xFFFF, 0xBEEF);
        assert_eq!(m.get8(0xFFFF), 0xEF, "wrong low byte");
        assert_eq!(m.get8(0x0000), 0xBE, "wrong high byte");
    }
}
