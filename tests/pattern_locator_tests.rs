use bit::analysis::{Pattern, PatternFormat, merge_matches};
use bitvec::prelude::*;

#[cfg(test)]
mod pattern_format_tests {
    use super::*;

    #[test]
    fn test_format_labels() {
        assert_eq!(PatternFormat::Bits.label(), "Bin");
        assert_eq!(PatternFormat::Octal.label(), "Oct");
        assert_eq!(PatternFormat::Hex.label(), "Hex");
        assert_eq!(PatternFormat::Ascii.label(), "ASCII");
    }

    #[test]
    fn test_format_all() {
        let all = PatternFormat::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], PatternFormat::Bits);
        assert_eq!(all[1], PatternFormat::Octal);
        assert_eq!(all[2], PatternFormat::Hex);
        assert_eq!(all[3], PatternFormat::Ascii);
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
    fn test_hex_parsing_without_prefix() {
        // Hex no longer requires 0x prefix
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Hex,
            "FF".to_string(),
            0,
        ).unwrap();

        assert_eq!(pattern.bits.len(), 8);
        assert!(pattern.bits.all());
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
mod octal_parsing_tests {
    use super::*;

    #[test]
    fn test_octal_parsing_basic() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "7".to_string(),
            0,
        ).unwrap();

        // 7 = 111 in binary = 3 bits
        assert_eq!(pattern.bits.len(), 3);
        assert!(pattern.bits.all());
    }

    #[test]
    fn test_octal_parsing_zero() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "0".to_string(),
            0,
        ).unwrap();

        assert_eq!(pattern.bits.len(), 3);
        assert!(pattern.bits.not_any());
    }

    #[test]
    fn test_octal_parsing_multiple_digits() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "377".to_string(),
            0,
        ).unwrap();

        // 3 digits * 3 bits = 9 bits
        assert_eq!(pattern.bits.len(), 9);
        // 3 = 011, 7 = 111, 7 = 111 -> 011111111
        assert_eq!(pattern.bits[0], false);
        assert_eq!(pattern.bits[1], true);
        assert_eq!(pattern.bits[2], true);
        assert!(pattern.bits[3..].iter().all(|b| *b));
    }

    #[test]
    fn test_octal_parsing_with_spaces() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "3 7 7".to_string(),
            0,
        ).unwrap();

        assert_eq!(pattern.bits.len(), 9);
    }

    #[test]
    fn test_octal_parsing_with_underscores() {
        let pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "3_7_7".to_string(),
            0,
        ).unwrap();

        assert_eq!(pattern.bits.len(), 9);
    }

    #[test]
    fn test_octal_parsing_error_invalid_digit() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "89".to_string(),
            0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid octal character"));
    }

    #[test]
    fn test_octal_parsing_error_empty() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "".to_string(),
            0,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Octal pattern is empty");
    }

    #[test]
    fn test_octal_parsing_error_only_whitespace() {
        let result = Pattern::new(
            "test".to_string(),
            PatternFormat::Octal,
            "   ".to_string(),
            0,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Octal pattern is empty");
    }

    #[test]
    fn test_octal_parsing_all_digits() {
        // Test all valid octal digits 0-7
        for digit in 0..=7u8 {
            let pattern = Pattern::new(
                "test".to_string(),
                PatternFormat::Octal,
                format!("{}", digit),
                0,
            ).unwrap();

            assert_eq!(pattern.bits.len(), 3);
            assert_eq!(pattern.bits[0], (digit & 4) != 0);
            assert_eq!(pattern.bits[1], (digit & 2) != 0);
            assert_eq!(pattern.bits[2], (digit & 1) != 0);
        }
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

        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        pattern.search(&haystack);

        assert_eq!(pattern.normal_matches.len(), 1);
        assert_eq!(pattern.normal_matches[0].position, 0);
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

        assert_eq!(pattern.normal_matches.len(), 3);
        assert_eq!(pattern.normal_matches[0].position, 0);
        assert_eq!(pattern.normal_matches[1].position, 2);
        assert_eq!(pattern.normal_matches[2].position, 4);
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

        assert_eq!(pattern.normal_matches.len(), 0);
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

        assert_eq!(pattern.normal_matches.len(), 2);
        assert_eq!(pattern.normal_matches[0].delta, None); // First match has no delta
        assert_eq!(pattern.normal_matches[1].delta, Some(4)); // Second match is 4 bits away
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

        assert_eq!(pattern.normal_matches.len(), 0);
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

        assert_eq!(pattern.normal_matches.len(), 0);
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

        assert_eq!(pattern.normal_matches.len(), 0);
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

        assert_eq!(pattern.normal_matches.len(), 1);
        assert_eq!(pattern.normal_matches[0].position, 0);
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

        assert_eq!(pattern.normal_matches.len(), 1);
        assert_eq!(pattern.normal_matches[0].position, 0);
    }
}

#[cfg(test)]
mod inverted_search_tests {
    use super::*;

    #[test]
    fn test_inverted_search_basic() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010".to_string(),
            0,
        ).unwrap();
        pattern.search_inverted = true;

        // Haystack contains the inverted pattern (0101) at position 0
        let haystack = bitvec![u8, Msb0; 0, 1, 0, 1];
        pattern.search(&haystack);

        assert_eq!(pattern.normal_matches.len(), 0); // No normal match
        assert_eq!(pattern.inverted_matches.len(), 1); // One inverted match
        assert_eq!(pattern.inverted_matches[0].position, 0);
    }

    #[test]
    fn test_inverted_search_both_match() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010".to_string(),
            0,
        ).unwrap();
        pattern.search_inverted = true;

        // Haystack contains normal at 0 and inverted at 4
        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0, 0, 1, 0, 1];
        pattern.search(&haystack);

        assert_eq!(pattern.normal_matches.len(), 1);
        assert_eq!(pattern.normal_matches[0].position, 0);
        assert_eq!(pattern.inverted_matches.len(), 1);
        assert_eq!(pattern.inverted_matches[0].position, 4);
    }

    #[test]
    fn test_inverted_search_disabled() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1010".to_string(),
            0,
        ).unwrap();
        pattern.search_inverted = false;

        let haystack = bitvec![u8, Msb0; 0, 1, 0, 1]; // Only inverted present
        pattern.search(&haystack);

        assert_eq!(pattern.normal_matches.len(), 0);
        assert_eq!(pattern.inverted_matches.len(), 0); // Not searched
    }

    #[test]
    fn test_inverted_all_ones_finds_all_zeros() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "1111".to_string(),
            0,
        ).unwrap();
        pattern.search_inverted = true;

        let haystack = bitvec![u8, Msb0; 0, 0, 0, 0];
        pattern.search(&haystack);

        assert_eq!(pattern.normal_matches.len(), 0);
        assert_eq!(pattern.inverted_matches.len(), 1);
        assert_eq!(pattern.inverted_matches[0].position, 0);
    }
}

#[cfg(test)]
mod merge_matches_tests {
    use super::*;

    #[test]
    fn test_merge_empty() {
        let patterns: Vec<Pattern> = Vec::new();
        let merged = merge_matches(&patterns);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_merge_single_pattern() {
        let mut pattern = Pattern::new(
            "test".to_string(),
            PatternFormat::Bits,
            "10".to_string(),
            0,
        ).unwrap();
        pattern.id = 1;

        let haystack = bitvec![u8, Msb0; 1, 0, 0, 0, 1, 0];
        pattern.search(&haystack);

        let merged = merge_matches(&[pattern]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].position, 0);
        assert_eq!(merged[0].pattern_id, 1);
        assert_eq!(merged[1].position, 4);
        assert_eq!(merged[1].pattern_id, 1);
    }

    #[test]
    fn test_merge_sorted_by_position() {
        let mut p1 = Pattern::new(
            "p1".to_string(),
            PatternFormat::Bits,
            "11".to_string(),
            0,
        ).unwrap();
        p1.id = 1;

        let mut p2 = Pattern::new(
            "p2".to_string(),
            PatternFormat::Bits,
            "00".to_string(),
            0,
        ).unwrap();
        p2.id = 2;

        // 11 00 11
        let haystack = bitvec![u8, Msb0; 1, 1, 0, 0, 1, 1];
        p1.search(&haystack);
        p2.search(&haystack);

        let merged = merge_matches(&[p1, p2]);
        // p1 at 0 and 4, p2 at 2
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].position, 0);
        assert_eq!(merged[0].pattern_id, 1);
        assert_eq!(merged[1].position, 2);
        assert_eq!(merged[1].pattern_id, 2);
        assert_eq!(merged[2].position, 4);
        assert_eq!(merged[2].pattern_id, 1);
    }

    #[test]
    fn test_merge_delta_next_same_id() {
        let mut p1 = Pattern::new(
            "p1".to_string(),
            PatternFormat::Bits,
            "1".to_string(),
            0,
        ).unwrap();
        p1.id = 1;

        // Bits: 1 0 1
        let haystack = bitvec![u8, Msb0; 1, 0, 1];
        p1.search(&haystack);

        let merged = merge_matches(&[p1]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].delta_next_same_id, Some(2)); // Next same-id match is 2 positions away
        assert_eq!(merged[1].delta_next_same_id, None); // Last match has no next
    }

    #[test]
    fn test_merge_skips_disabled() {
        let mut p1 = Pattern::new(
            "p1".to_string(),
            PatternFormat::Bits,
            "10".to_string(),
            0,
        ).unwrap();
        p1.id = 1;
        p1.enabled = false; // Disabled!

        let haystack = bitvec![u8, Msb0; 1, 0, 1, 0];
        p1.search(&haystack);

        let merged = merge_matches(&[p1]);
        assert!(merged.is_empty()); // Disabled pattern's matches are excluded
    }

    #[test]
    fn test_merge_includes_inverted() {
        let mut p1 = Pattern::new(
            "p1".to_string(),
            PatternFormat::Bits,
            "10".to_string(),
            0,
        ).unwrap();
        p1.id = 1;
        p1.search_inverted = true;

        // 10 01 - normal at 0, inverted (01) at 2
        let haystack = bitvec![u8, Msb0; 1, 0, 0, 1];
        p1.search(&haystack);

        let merged = merge_matches(&[p1]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].position, 0);
        assert!(!merged[0].is_inverted);
        assert_eq!(merged[1].position, 2);
        assert!(merged[1].is_inverted);
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

        pattern.input = "GG".to_string(); // Invalid hex chars
        let result = pattern.update_bits();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn test_validate_success() {
        let mut pattern = Pattern::new_empty(1);
        pattern.format = PatternFormat::Bits;
        pattern.input = "1010".to_string();
        pattern.validate();

        assert!(pattern.validation_error.is_none());
        assert_eq!(pattern.bits.len(), 4);
    }

    #[test]
    fn test_validate_error() {
        let mut pattern = Pattern::new_empty(1);
        pattern.format = PatternFormat::Bits;
        pattern.input = "ZZZZ".to_string();
        pattern.validate();

        assert!(pattern.validation_error.is_some());
    }

    #[test]
    fn test_validate_empty_input() {
        let mut pattern = Pattern::new_empty(1);
        pattern.format = PatternFormat::Bits;
        pattern.input = String::new();
        pattern.validate();

        assert!(pattern.validation_error.is_none());
        assert!(pattern.bits.is_empty());
    }
}

#[cfg(test)]
mod pattern_locator_state_tests {
    use bit::analysis::PatternLocatorState;

    #[test]
    fn test_default_has_4_rows() {
        let state = PatternLocatorState::default();
        assert_eq!(state.patterns.len(), 4);
        assert_eq!(state.patterns[0].id, 1);
        assert_eq!(state.patterns[3].id, 4);
        assert_eq!(state.next_id, 5);
    }

    #[test]
    fn test_add_and_remove_pattern() {
        let mut state = PatternLocatorState::default();
        // Default has 4 rows
        let initial = state.patterns.len();

        state.add_pattern();
        assert_eq!(state.patterns.len(), initial + 1);
        assert_eq!(state.patterns[initial].id, 5);

        state.add_pattern();
        assert_eq!(state.patterns.len(), initial + 2);
        assert_eq!(state.patterns[initial + 1].id, 6);

        state.remove_pattern(0);
        assert_eq!(state.patterns.len(), initial + 1);
    }

    #[test]
    fn test_duplicate_pattern() {
        let mut state = PatternLocatorState::default();
        state.patterns[0].name = "Original".to_string();
        state.patterns[0].input = "1010".to_string();

        let count_before = state.patterns.len();
        state.duplicate_pattern(0);
        assert_eq!(state.patterns.len(), count_before + 1);
        assert_eq!(state.patterns[1].name, "Original (copy)");
        assert_eq!(state.patterns[1].input, "1010");
        assert_ne!(state.patterns[0].id, state.patterns[1].id);
    }

    #[test]
    fn test_set_all_enabled() {
        let mut state = PatternLocatorState::default();

        state.set_all_enabled(false);
        assert!(state.patterns.iter().all(|p| !p.enabled));

        state.set_all_enabled(true);
        assert!(state.patterns.iter().all(|p| p.enabled));
    }
}
