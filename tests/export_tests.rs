#[test]
fn test_hex_dump_format() {
    // Test that hex dump formatting works correctly
    let data: Vec<u8> = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
                            0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

    let hex_dump = data.chunks(16)
        .enumerate()
        .map(|(i, chunk)| {
            let hex: String = chunk.iter()
                .map(|b| format!("{:02x} ", b))
                .collect();
            let ascii: String = chunk.iter()
                .map(|&b| if (32u8..127u8).contains(&b) { b as char } else { '.' })
                .collect();
            format!("{:08x}  {:<48} {}", i * 16, hex, ascii)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Verify format has address, hex bytes, and ascii representation
    assert!(hex_dump.contains("00000000"));
    assert!(hex_dump.contains("00 11 22 33"));
}

#[test]
fn test_base64_encoding() {
    // Test that base64 encoding works correctly
    use base64::Engine;
    let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"
    let base64_str = base64::engine::general_purpose::STANDARD.encode(&data);

    // "Hello" in base64 should be "SGVsbG8="
    assert_eq!(base64_str, "SGVsbG8=");
}

#[test]
fn test_hex_dump_with_printable_ascii() {
    let data: &[u8] = b"Hello World!";

    let chunk = data;
    let ascii: String = chunk.iter()
        .map(|&b| if (32u8..127u8).contains(&b) { b as char } else { '.' })
        .collect();

    assert_eq!(ascii, "Hello World!");
}

#[test]
fn test_hex_dump_with_non_printable() {
    let data: Vec<u8> = vec![0x00, 0x01, 0x48, 0x65, 0x6C, 0x6C, 0x6F]; // \0\1Hello

    let chunk = data.as_slice();
    let ascii: String = chunk.iter()
        .map(|&b| if (32u8..127u8).contains(&b) { b as char } else { '.' })
        .collect();

    assert_eq!(ascii, "..Hello");
}
