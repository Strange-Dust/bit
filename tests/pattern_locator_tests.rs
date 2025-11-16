use bit::analysis::{Pattern, PatternFormat};
use bitvec::prelude::*;

#[cfg(test)]
mod pattern_format_tests {
    use super::*;

    #[test]
    fn test_format_names() {
        assert_eq!(PatternFormat::Hex.name(), "Hex (0x...)");
        assert_eq!(PatternFormat::Ascii.name(), "ASCII");
        assert_eq!(PatternFormat::Bits.name(), "Bits (0/1)");
    }
}

#[cfg(test)]
mod hex_parsing_tests {
    use super::*;

    #[test]
    fn test_hex_parsing_basic() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xFF".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        assert!(pattern.bits.all());
    }

    #[test]
    fn test_hex_parsing_zero() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0x00".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        assert!(pattern.bits.not_any());
    }

    #[test]
    fn test_hex_parsing_multiple_bytes() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xDEADBEEF".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 32);
    }

    #[test]
    fn test_hex_parsing_lowercase() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xabcd".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 16);
    }

    #[test]
    fn test_hex_parsing_uppercase_prefix() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0XFF".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
    }

    #[test]
    fn test_hex_parsing_with_whitespace() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "  0xFF  ".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
    }

    #[test]
    fn test_hex_parsing_error_no_prefix() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "FF".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Hex pattern must start with 0x");
    }

    #[test]
    fn test_hex_parsing_error_empty() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0x".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Hex pattern is empty");
    }

    #[test]
    fn test_hex_parsing_error_invalid_char() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xGG".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex character"));
    }

    #[test]
    fn test_hex_parsing_error_special_chars() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0x!@#$".to_string(),
            0,
        );
        
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod ascii_parsing_tests {
    use super::*;

    #[test]
    fn test_ascii_parsing_basic() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "A".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        // 'A' is 0x41 = 01000001
        assert_eq!(pattern.bits[0], false);
        assert_eq!(pattern.bits[1], true);
    }

    #[test]
    fn test_ascii_parsing_multiple_chars() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "ABC".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 24); // 3 chars * 8 bits
    }

    #[test]
    fn test_ascii_parsing_special_chars() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "!@#$".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 32);
    }

    #[test]
    fn test_ascii_parsing_numbers() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "123".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 24);
    }

    #[test]
    fn test_ascii_parsing_whitespace() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "A B".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 24); // 3 chars including space
    }

    #[test]
    fn test_ascii_parsing_error_empty() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "ASCII pattern is empty");
    }
}

#[cfg(test)]
mod bits_parsing_tests {
    use super::*;

    #[test]
    fn test_bits_parsing_basic() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10101010".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        assert_eq!(pattern.bits[0], true);
        assert_eq!(pattern.bits[1], false);
    }

    #[test]
    fn test_bits_parsing_all_ones() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "11111111".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        assert!(pattern.bits.all());
    }

    #[test]
    fn test_bits_parsing_all_zeros() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "00000000".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
        assert!(pattern.bits.not_any());
    }

    #[test]
    fn test_bits_parsing_with_spaces() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010 1010".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8); // Spaces ignored
    }

    #[test]
    fn test_bits_parsing_with_underscores() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010_1010".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8); // Underscores ignored
    }

    #[test]
    fn test_bits_parsing_with_whitespace() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "  10101010  ".to_string(),
            0,
        ).unwrap();
        
        assert_eq!(pattern.bits.len(), 8);
    }

    #[test]
    fn test_bits_parsing_error_empty() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Bit pattern is empty");
    }

    #[test]
    fn test_bits_parsing_error_only_whitespace() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "   ".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Bit pattern is empty");
    }

    #[test]
    fn test_bits_parsing_error_invalid_char() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10102010".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid bit character: 2"));
    }

    #[test]
    fn test_bits_parsing_error_letters() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "101A1010".to_string(),
            0,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid bit character"));
    }
}

#[cfg(test)]
mod pattern_search_tests {
    use super::*;

    #[test]
    fn test_exact_match_single() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010".to_string(),
            0,
        ).unwrap();
        
        let mut haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 1);
        assert_eq!(pattern.matches[0].position, 0);
        assert_eq!(pattern.matches[0].mismatches, 0);
    }

    #[test]
    fn test_exact_match_multiple() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10".to_string(),
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 3);
        assert_eq!(pattern.matches[0].position, 0);
        assert_eq!(pattern.matches[1].position, 2);
        assert_eq!(pattern.matches[2].position, 4);
    }

    #[test]
    fn test_no_match() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1111".to_string(),
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 0, 0, 0, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 0);
    }

    #[test]
    fn test_match_with_one_garble() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1111".to_string(),
            1, // Allow 1 mismatch
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 1, 1, 0]; // Last bit differs
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 1);
        assert_eq!(pattern.matches[0].position, 0);
        assert_eq!(pattern.matches[0].mismatches, 1);
    }

    #[test]
    fn test_match_with_multiple_garbles() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "11111111".to_string(),
            2, // Allow 2 mismatches
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 1, 0, 1, 1, 0, 1, 1]; // 2 bits differ
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 1);
        assert_eq!(pattern.matches[0].position, 0);
        assert_eq!(pattern.matches[0].mismatches, 2);
    }

    #[test]
    fn test_no_match_too_many_garbles() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1111".to_string(),
            1, // Allow only 1 mismatch
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 1, 0, 0]; // 2 bits differ
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 0);
    }

    #[test]
    fn test_delta_calculation() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "11".to_string(),
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 1, 0, 0, 1, 1];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 2);
        assert_eq!(pattern.matches[0].delta, None); // First match has no delta
        assert_eq!(pattern.matches[1].delta, Some(4)); // Second match is 4 bits away
    }

    #[test]
    fn test_empty_pattern() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1".to_string(),
            0,
        ).unwrap();
        pattern.bits.clear();
        
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 0);
    }

    #[test]
    fn test_empty_haystack() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10".to_string(),
            0,
        ).unwrap();
        
        let haystack = BitVec::<u8, Msb0>::new();
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 0);
    }

    #[test]
    fn test_pattern_longer_than_haystack() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10101010".to_string(),
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 0);
    }

    #[test]
    fn test_hex_pattern_search() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xAA".to_string(), // 10101010
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 1);
        assert_eq!(pattern.matches[0].position, 0);
    }

    #[test]
    fn test_ascii_pattern_search() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "A".to_string(), // 0x41 = 01000001
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 0, 1, 0, 0, 0, 0, 0, 1];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches.len(), 1);
        assert_eq!(pattern.matches[0].position, 0);
    }
}

#[cfg(test)]
mod pattern_match_tests {
    use super::*;

    #[test]
    fn test_bits_string() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010".to_string(),
            0,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches[0].bits_string(), "1010");
    }

    #[test]
    fn test_bits_string_with_garbles() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1111".to_string(),
            2,
        ).unwrap();
        
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        pattern.search(&haystack);
        
        assert_eq!(pattern.matches[0].bits_string(), "1010"); // Shows actual matched bits
    }
}

#[cfg(test)]
mod update_bits_tests {
    use super::*;

    #[test]
    fn test_update_bits_hex() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xFF".to_string(),
            0,
        ).unwrap();
        
        pattern.input = "0x00".to_string();
        pattern.update_bits().unwrap();
        
        assert!(pattern.bits.not_any());
    }

    #[test]
    fn test_update_bits_ascii() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Ascii,
            "A".to_string(),
            0,
        ).unwrap();
        
        pattern.input = "AB".to_string();
        pattern.update_bits().unwrap();
        
        assert_eq!(pattern.bits.len(), 16);
    }

    #[test]
    fn test_update_bits_error() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "0xFF".to_string(),
            0,
        ).unwrap();
        
        pattern.input = "FF".to_string(); // Missing 0x prefix
        let result = pattern.update_bits();
        
        assert!(result.is_err());
    }
}
