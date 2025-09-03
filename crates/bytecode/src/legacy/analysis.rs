use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::vec::Vec;

/// Analyzes the bytecode for use in [`LegacyAnalyzedBytecode`](crate::LegacyAnalyzedBytecode).
///
/// See [`LegacyAnalyzedBytecode`](crate::LegacyAnalyzedBytecode) for more details.
///
/// Prefer using [`LegacyAnalyzedBytecode::analyze`](crate::LegacyAnalyzedBytecode::analyze) instead.
pub fn analyze_legacy(bytecode: Bytes) -> (JumpTable, Bytes) {
    if bytecode.is_empty() {
        return (JumpTable::default(), Bytes::from_static(&[opcode::STOP]));
    }

    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let range = bytecode.as_ptr_range();
    let start = range.start;
    let mut iterator = start;
    let end = range.end;
    let mut opcode = 0;

    while iterator < end {
        opcode = unsafe { *iterator };

        // for si
        if let Some(steps) = unsafe {code_bitmap_for_si(&mut jumps, opcode, iterator.offset_from(start) as usize) } {
            iterator = unsafe { iterator.add(steps) };
            continue
        }
        // end for si

        if opcode == opcode::JUMPDEST {
            // SAFETY: Jumps are max length of the code
            unsafe { jumps.set_unchecked(iterator.offset_from(start) as usize, true) }
            iterator = unsafe { iterator.add(1) };
        } else {
            let push_offset = opcode.wrapping_sub(opcode::PUSH1);
            if push_offset < 32 {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.add(push_offset as usize + 2) };
            } else {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.add(1) };
            }
        }
    }

    let padding = (iterator as usize) - (end as usize) + (opcode != opcode::STOP) as usize;
    let bytecode = if padding > 0 {
        let mut padded = Vec::with_capacity(bytecode.len() + padding);
        padded.extend_from_slice(&bytecode);
        padded.resize(padded.len() + padding, 0);
        Bytes::from(padded)
    } else {
        bytecode
    };

    (JumpTable::new(jumps), bytecode)
}

#[warn(unused_unsafe)]
fn code_bitmap_for_si(_jumps: &mut BitVec<u8, Lsb0>, code: u8, _pos: usize) -> Option<usize> {
    match code {
        opcode::PUSH2JUMP | opcode::PUSH2JUMPI => {Some(4)}

        opcode::PUSH1PUSH1 => {Some(4)}

        opcode::PUSH1ADD | opcode::PUSH1SHL | opcode::PUSH1DUP1 => {Some(3)}

        opcode::JUMPIFZERO => {Some(5)}

        opcode::ISZEROPUSH2 => {Some(4)}

        opcode::DUP2MSTOREPUSH1ADD => {Some(5)}

        opcode::DUP1PUSH4EQPUSH2 => {Some(10)}

        opcode::PUSH1CALLDATALOADPUSH1SHRDUP1PUSH4GTPUSH2 => {Some(16)}

        opcode::PUSH1PUSH1PUSH1SHLSUB => {Some(8)}

        opcode:: SWAP1PUSH1DUP1NOTSWAP2ADDANDDUP2ADDSWAP1DUP2LT => {Some(13)}

        opcode::DUP3AND => {
            Some(2)
        }

        opcode::SWAP2SWAP1DUP3SUBSWAP2DUP3GTPUSH2 => {
            // set2(jumps, pos+7);
            Some(10)
        }

        opcode::SWAP1DUP2 => {
            Some(2)
        }

        opcode::SHRSHRDUP1MULDUP1 => {
            Some(5)
        }

        opcode::SWAP3POPPOPPOP => {
            Some(4)
        }

        opcode::SUBSLTISZEROPUSH2 => {
            Some(6)
        }

        opcode::DUP11MULDUP3SUBMULDUP1 => {
            Some(6)
        }

        _ => {None}
    }
}

#[cfg(test)]
mod tests {
    use primitives::hex::FromHex;

    use super::*;

    #[test]
    fn test_analyze_legacy() {
        let (jp1, bt1) = analyze_legacy(Bytes::from_hex("82b5016eb05b6101b386").unwrap());
        // println!("{:?}, {:?}", jp, bt);

        let (jp2, bt2) = analyze_legacy(Bytes::from_hex("8261016e565b6101b386").unwrap());
        // println!("{:?}, {:?}", jp, bt);
        assert_eq!(jp1, jp2);
        assert_ne!(bt1, bt2);
    }

    #[test]
    fn test_bytecode_ends_with_stop_no_padding_needed() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
    }

    #[test]
    fn test_bytecode_ends_without_stop_requires_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH1, 0x02, opcode::ADD];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 1);
    }

    #[test]
    fn test_bytecode_ends_with_push16_requires_17_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH16];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 17);
    }

    #[test]
    fn test_bytecode_ends_with_push2_requires_2_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH2, 0x02];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 2);
    }

    #[test]
    fn test_empty_bytecode_requires_stop() {
        let bytecode = vec![];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), 1); // Just STOP
    }

    #[test]
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.is_valid(0)); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.is_valid(2)); // JUMPDEST should be at position 2
    }

    #[test]
    fn test_bytecode_with_multiple_jumpdests() {
        let bytecode = vec![
            opcode::JUMPDEST,
            opcode::PUSH1,
            0x01,
            opcode::JUMPDEST,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.is_valid(0)); // First JUMPDEST
        assert!(jump_table.is_valid(3)); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(!jump_table.is_valid(0)); // Invalid opcode should not be a jumpdest
    }

    #[test]
    fn test_bytecode_with_sequential_pushes() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH2,
            0x02,
            0x03,
            opcode::PUSH4,
            0x04,
            0x05,
            0x06,
            0x07,
            opcode::STOP,
        ];
        let (jump_table, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
        assert!(!jump_table.is_valid(0)); // PUSH1
        assert!(!jump_table.is_valid(2)); // PUSH2
        assert!(!jump_table.is_valid(5)); // PUSH4
    }

    #[test]
    fn test_bytecode_with_jumpdest_in_push_data() {
        let bytecode = vec![
            opcode::PUSH2,
            opcode::JUMPDEST, // This should not be treated as a JUMPDEST
            0x02,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(!jump_table.is_valid(1)); // JUMPDEST in push data should not be valid
    }

    #[test]
    fn test_analyze_legacy_with_fused_instructions() {
        // Test that analyze_legacy correctly processes fused instructions
        
        // Test with DUP3AND (should skip 2 bytes) - when iterator goes past end, padding is added
        let bytecode = vec![opcode::DUP3AND, opcode::STOP]; // 2 bytes total
        let (_, analyzed_bytecode) = analyze_legacy(bytecode.clone().into());
        // DUP3AND skips 2 bytes, so iterator moves past STOP, causing padding
        assert_eq!(analyzed_bytecode.len(), 3); // Original 2 + 1 padding = 3
        
        // Test with SWAP1DUP2 (should skip 2 bytes)  
        let bytecode = vec![opcode::SWAP1DUP2, opcode::STOP]; // 2 bytes total
        let (_, analyzed_bytecode) = analyze_legacy(bytecode.clone().into());
        // SWAP1DUP2 skips 2 bytes, so iterator moves past STOP, causing padding
        assert_eq!(analyzed_bytecode.len(), 3); // Original 2 + 1 padding = 3

        // Test with SHRSHRDUP1MULDUP1 (should skip 5 bytes)
        let bytecode = vec![opcode::SHRSHRDUP1MULDUP1, 0, 0, 0, 0, opcode::STOP]; // 6 bytes total
        let (_, analyzed_bytecode) = analyze_legacy(bytecode.clone().into());
        // SHRSHRDUP1MULDUP1 skips 5 bytes, ends exactly at STOP, no padding needed
        assert_eq!(analyzed_bytecode.len(), 6); // No padding needed

        // Test incomplete fused instruction (should add padding)
        let bytecode = vec![opcode::SHRSHRDUP1MULDUP1]; // Missing 4 bytes + STOP
        let (_, analyzed_bytecode) = analyze_legacy(bytecode.clone().into());
        // SHRSHRDUP1MULDUP1 tries to skip 5 bytes but only 1 available, needs 4 more + STOP
        assert_eq!(analyzed_bytecode.len(), 6); // Original 1 + 4 padding + 1 STOP = 6
    }

    #[test]
    fn test_fused_instructions_consistency() {
        // This test verifies that the bitmap step values are consistent
        // with what the actual functions should consume
        
        let consistency_tests = [
            // (opcode, bitmap_steps, description)
            (opcode::DUP3AND, 2, "DUP3(1) + AND(1) = 2 bytes"),
            (opcode::SWAP1DUP2, 2, "SWAP1(1) + DUP2(1) = 2 bytes"),
            (opcode::SHRSHRDUP1MULDUP1, 5, "SHR(1) + SHR(1) + DUP1(1) + MUL(1) + DUP1(1) = 5 bytes"),
            (opcode::SWAP3POPPOPPOP, 4, "SWAP3(1) + POP(1) + POP(1) + POP(1) = 4 bytes"),
            (opcode::DUP11MULDUP3SUBMULDUP1, 6, "DUP11(1) + MUL(1) + DUP3(1) + SUB(1) + MUL(1) + DUP1(1) = 6 bytes"),
            (opcode::SWAP2SWAP1DUP3SUBSWAP2DUP3GTPUSH2, 10, "7 ops(7) + PUSH2(1) + immediate(2) = 10 bytes"),
            (opcode::SUBSLTISZEROPUSH2, 5, "SUB(1) + SLT(1) + ISZERO(1) + PUSH2(1) + immediate(2) = 6 bytes, but bitmap shows 5 - CHECK THIS!"),
        ];

        use bitvec::{bitvec, order::Lsb0};
        let mut jumps = bitvec![u8, Lsb0; 0; 100];
        
        for (opcode_val, expected_steps, description) in consistency_tests {
            let result = code_bitmap_for_si(&mut jumps, opcode_val, 0);
            println!("Testing 0x{:02X}: {} -> {:?}", opcode_val, description, result);
            
            assert_eq!(
                result,
                Some(expected_steps),
                "Opcode 0x{:02X} ({}): expected {}, got {:?}",
                opcode_val, description, expected_steps, result
            );
        }
    }
}
