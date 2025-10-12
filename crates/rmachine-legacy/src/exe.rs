use anyhow::{Result, bail};

use std::path::Path;

use crate::asm::{Image, Word, assemble};

/// Builds an executable from `input`.
///
/// # Format
///
/// The executable format consists of:
///
/// 1. **Magic number** (4 bytes): ASCII string "rme1"
/// 2. **Image data**: A flat sequence of 32-bit words in big-endian format
///
/// Each word from the assembled `Image` is serialized sequentially as 4 bytes.
/// The image contains both code and data interleaved as a single contiguous
/// sequence without section headers or delimiters.
///
/// # Example
///
/// For an image containing the RISC-V instruction `addi a0, zero, 1` (from `li a0, 1`):
/// ```text
/// Offset  Bytes           Description
/// 0x00    72 6D 65 31     Magic: "rme1"
/// 0x04    00 10 05 13     Word: 0x00100513 (addi a0, zero, 1)
/// ```
///
/// # Errors
///
/// Returns any errors reading the input, assembling the program,
/// or writing the executable to disk.
pub fn build_exe(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<()> {
    const HEADER: &[u8] = b"rme1";

    let source = std::fs::read_to_string(input)?;
    let image = assemble(&source)?;
    let mut bytes = Vec::from(HEADER);
    bytes.extend(image.into_iter().flat_map(Word::to_be_bytes));

    std::fs::write(output, bytes)?;
    Ok(())
}

/// Creates a memory image from a byte slice.
///
/// # Format
///
/// Expects a byte slice in the executable format:
///
/// 1. **Magic number** (4 bytes): ASCII string "rme1"
/// 2. **Image data**: A flat sequence of 32-bit words in big-endian format
///
/// Each 4-byte chunk after the magic number is parsed as a big-endian word
/// and added to the resulting `Image`.
///
/// # Errors
///
/// Returns an error if the magic number `rme1` is not found at the beginning.
pub fn try_image_from_bytes(bytes: &[u8]) -> Result<Image> {
    let mut offset = 0;

    // Read the magic number.
    let header = &bytes[offset..4];
    if header != b"rme1" {
        bail!("Invalid magic number {header:?}");
    }
    offset += 4;

    let mut image = Vec::new();
    let text_chunks = bytes[offset..].chunks(4);
    for chunk in text_chunks {
        let word = Word::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        image.push(word);
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn build_exe_fn_creates_an_executable_from_an_asm_source_file() {
        let mut dir = tempdir().unwrap();
        let mut exe_path = dir.path().to_owned();
        exe_path.push("test");

        build_exe("testdata/li.s", exe_path.clone()).unwrap();

        // Magic "rme1" + word 0x00100513 in big-endian
        let want = vec![b'r', b'm', b'e', b'1', 0, 16, 5, 19];
        let got = std::fs::read(exe_path).unwrap();
        assert_eq!(want, got, "wrong bytes");
    }

    #[test]
    fn try_image_from_bytes_fn_returns_ok_for_valid_magic_number() {
        let bytes = vec![b'r', b'm', b'e', b'1', 0, 0, 0, 0];
        let image = try_image_from_bytes(&bytes).unwrap();
        assert_eq!(image, vec![0]);
    }

    #[test]
    fn try_image_from_bytes_fn_returns_error_for_invalid_magic_number() {
        let bytes = vec![b'r', b'm', b'e', b'2', 0, 0, 0, 0];
        let result = try_image_from_bytes(&bytes);
        assert!(result.is_err());
    }
}
