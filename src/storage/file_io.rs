use bitvec::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc::Sender;

/// Maximum file size to read (1 GB)
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;

/// Progress callback for file loading
pub enum LoadProgress {
    Progress { loaded: u64, total: u64 },
    Complete(Result<BitVec<u8, Msb0>, String>),
}

/// Read a file and convert its contents to a bit vector with progress reporting
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The file cannot be opened
/// - The file is larger than MAX_FILE_SIZE
/// - Reading the file fails
pub fn read_file_as_bits_with_progress(
    path: &Path,
    progress_tx: Sender<LoadProgress>,
) -> std::io::Result<()> {
    let result = (|| -> std::io::Result<BitVec<u8, Msb0>> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let total_size = metadata.len();
        
        // Check file size
        if total_size > MAX_FILE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("File too large: {} bytes (max {} bytes)", total_size, MAX_FILE_SIZE)
            ));
        }
        
        let mut file = file;
        let mut buffer = Vec::new();
        let chunk_size = 1024 * 1024; // 1MB chunks
        let mut total_read: u64 = 0;
        
        loop {
            let mut chunk = vec![0u8; chunk_size];
            match file.read(&mut chunk)? {
                0 => break,
                n => {
                    buffer.extend_from_slice(&chunk[..n]);
                    total_read += n as u64;
                    
                    // Send progress update
                    let _ = progress_tx.send(LoadProgress::Progress {
                        loaded: total_read,
                        total: total_size,
                    });
                }
            }
        }
        
        let bits = BitVec::<u8, Msb0>::from_vec(buffer);
        Ok(bits)
    })();
    
    // Send completion message
    let _ = progress_tx.send(LoadProgress::Complete(
        result.map_err(|e| e.to_string())
    ));
    
    Ok(())
}

/// Read a file and convert its contents to a bit vector
/// 
/// # Errors
/// 
/// Returns an error if:
/// - The file cannot be opened
/// - The file is larger than MAX_FILE_SIZE
/// - Reading the file fails
pub fn read_file_as_bits(path: &Path) -> std::io::Result<BitVec<u8, Msb0>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    
    // Check file size
    if metadata.len() > MAX_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("File too large: {} bytes (max {} bytes)", metadata.len(), MAX_FILE_SIZE)
        ));
    }
    
    let mut file = file;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let bits = BitVec::<u8, Msb0>::from_vec(buffer);
    Ok(bits)
}

pub fn write_bits_to_file(path: &Path, bits: &BitVec<u8, Msb0>) -> std::io::Result<()> {
    if bits.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot write empty bit vector to file"
        ));
    }
    
    let mut file = File::create(path)?;
    
    // Convert bits to bytes
    // If the bit count is not a multiple of 8, pad with zeros
    let byte_vec = bits.clone().into_vec();
    
    file.write_all(&byte_vec)?;
    file.flush()?;
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
    
    #[test]
    fn test_read_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let bits = read_file_as_bits(temp_file.path()).unwrap();
        assert_eq!(bits.len(), 0);
    }
    
    #[test]
    fn test_read_single_byte() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0xFF]).unwrap();
        temp_file.flush().unwrap();
        
        let bits = read_file_as_bits(temp_file.path()).unwrap();
        assert_eq!(bits.len(), 8);
        assert!(bits.all());
    }
    
    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file_as_bits(Path::new("nonexistent_file.bin"));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_write_empty_bits() {
        let temp_file = NamedTempFile::new().unwrap();
        let empty_bits = BitVec::<u8, Msb0>::new();
        let result = write_bits_to_file(temp_file.path(), &empty_bits);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_write_single_bit() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut bits = BitVec::<u8, Msb0>::new();
        bits.push(true);
        
        write_bits_to_file(temp_file.path(), &bits).unwrap();
        
        let mut data = Vec::new();
        File::open(temp_file.path())
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        
        // Single bit true, padded to byte = 10000000 = 0x80
        assert_eq!(data, vec![0x80]);
    }
    
    #[test]
    fn test_bit_preservation() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut bits = BitVec::<u8, Msb0>::new();
        
        // Create specific pattern: 10101010 11110000
        for &b in &[true, false, true, false, true, false, true, false] {
            bits.push(b);
        }
        for &b in &[true, true, true, true, false, false, false, false] {
            bits.push(b);
        }
        
        write_bits_to_file(temp_file.path(), &bits).unwrap();
        let read_bits = read_file_as_bits(temp_file.path()).unwrap();
        
        assert_eq!(bits, read_bits);
    }
}
