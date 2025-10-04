use anyhow::{Result, bail};

use std::path::Path;

use crate::asm::{Image, Word, assemble};

/// Builds an executable from `input`.
///
/// # Errors
///
/// Returns any errors reading the input, assembling the program
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
/// # Errors
///
/// Returns an error if the magic number is not found.
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

        // header + text section length + text section + data section length + data section
        let want = vec![b'r', b'm', b'e', b'1', 0, 16, 5, 19];
        let got = std::fs::read(exe_path).unwrap();
        assert_eq!(want, got, "wrong bytes");
    }

    #[test]
    fn we_can_turn_bytes_into_image() {
        let bytes = vec![b'r', b'm', b'e', b'1', 0, 0, 0, 0];
        let image = try_image_from_bytes(&bytes).unwrap();
        assert_eq!(image, vec![0]);
    }
}
