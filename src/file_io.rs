use bitvec::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn read_file_as_bits(path: &Path) -> std::io::Result<BitVec<u8, Msb0>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let bits = BitVec::<u8, Msb0>::from_vec(buffer);
    Ok(bits)
}

pub fn write_bits_to_file(path: &Path, bits: &BitVec<u8, Msb0>) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    
    // Convert bits to bytes
    // If the bit count is not a multiple of 8, pad with zeros
    let byte_vec = bits.clone().into_vec();
    
    file.write_all(&byte_vec)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_roundtrip() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = vec![0b10101010, 0b11110000, 0b00001111];
        temp_file.write_all(&test_data).unwrap();
        temp_file.flush().unwrap();

        let bits = read_file_as_bits(temp_file.path()).unwrap();
        assert_eq!(bits.len(), 24);

        let output_file = NamedTempFile::new().unwrap();
        write_bits_to_file(output_file.path(), &bits).unwrap();

        let mut output_data = Vec::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_end(&mut output_data)
            .unwrap();

        assert_eq!(output_data, test_data);
    }
}
