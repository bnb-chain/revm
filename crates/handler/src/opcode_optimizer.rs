use thiserror::Error;
use bytecode::opcode as op;

/// superinstruction in revm

pub(crate) const MIN_OPTIMIZED_OPCODE: u8 = 0xB0;
/// superinstruction max opcode
pub(crate) const MAX_OPTIMIZED_OPCODE: u8 = 0xCF;

/// FailPreprocessing Fusion err
#[derive(Debug, Error)]
pub(crate) enum FusionError {
    #[error("optimized opcode already present (pre-processing fail)")]
    /// Input bytecode already contains optimized opcodes; fusion aborted.
    FailPreprocessing,
}

pub(crate) fn do_code_fusion(code: &[u8]) -> Result<Vec<u8>, FusionError> {
    // return Ok(code.to_vec());
    let mut fused = code.to_vec();
    let mut i = 0usize;
    while i < fused.len() {
        let cur = i;

        if fused[cur] == op::INVALID {
            return Ok(fused);
        }

        if fused[cur] <= MIN_OPTIMIZED_OPCODE && fused[cur] >= MAX_OPTIMIZED_OPCODE {
            return Err(FusionError::FailPreprocessing);
        }

        // ----------------------------
        // 15-byte 
        // PUSH1 _ CALLDATALOAD PUSH1 _ SHR DUP1 PUSH4 _ _ _ _ _ GT PUSH2 _ _
        // ----------------------------
        if cur + 15 < fused.len() {
            let c = |o: usize| fused[cur + o];
            if c(0) == op::PUSH1
                && c(2) == op::CALLDATALOAD
                && c(3) == op::PUSH1
                && c(5) == op::SHR
                && c(6) == op::DUP1
                && c(7) == op::PUSH4
                && c(12) == op::GT
                && c(13) == op::PUSH2
            {
                fused[cur] = op::PUSH1CALLDATALOADPUSH1SHRDUP1PUSH4GTPUSH2;
                for off in [2, 3, 5, 6, 7, 12, 13] {
                    fused[cur + off] = op::SNOP;
                }
                i += 16;
                continue;
            }
        }

        // ----------------------------
        // 12-byte 
        // SWAP1 PUSH1 _ DUP1 NOT SWAP2 ADD AND DUP2 ADD SWAP1 DUP2 LT
        // ----------------------------
        if cur + 12 < fused.len() {
            let c = |o: usize| fused[cur + o];
            if c(0) == op::SWAP1
                && c(1) == op::PUSH1
                && c(3) == op::DUP1
                && c(4) == op::NOT
                && c(5) == op::SWAP2
                && c(6) == op::ADD
                && c(7) == op::AND
                && c(8) == op::DUP2
                && c(9) == op::ADD
                && c(10) == op::SWAP1
                && c(11) == op::DUP2
                && c(12) == op::LT
            {
                fused[cur] = op::SWAP1PUSH1DUP1NOTSWAP2ADDANDDUP2ADDSWAP1DUP2LT;
                for off in [1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] {
                    fused[cur + off] = op::SNOP;
                }
                i += 13;
                continue;
            }
        }

        // ----------------------------
        // 9-byte 
        // DUP1 PUSH4 _ _ _ _ EQ PUSH2 _ _
        // ----------------------------
        if cur + 9 < fused.len() {
            let c = |o: usize| fused[cur + o];
            if c(0) == op::DUP1 && c(1) == op::PUSH4 && c(6) == op::EQ && c(7) == op::PUSH2 {
                fused[cur] = op::DUP1PUSH4EQPUSH2;
                for off in [1, 6, 7] {
                    fused[cur + off] = op::SNOP;
                }
                i += 10;
                continue;
            }
        }

        // ----------------------------
        // 7-byte 
        // PUSH1 _ PUSH1 _ PUSH1 _ SHL SUB
        // ----------------------------
        if cur + 7 < fused.len() {
            let c = |o: usize| fused[cur + o];
            if c(0) == op::PUSH1
                && c(2) == op::PUSH1
                && c(4) == op::PUSH1
                && c(6) == op::SHL
                && c(7) == op::SUB
            {
                fused[cur] = op::PUSH1PUSH1PUSH1SHLSUB;
                for off in [2, 4, 6, 7] {
                    fused[cur + off] = op::SNOP;
                }
                i += 8;
                continue;
            }
        }

        // ----------------------------
        // 6-byte (AND DUP2 ADD SWAP1 DUP2 LT)
        // ----------------------------
        if cur + 5 < fused.len() {
            let c = |o: usize| fused[cur + o];
            if c(0) == op::AND
                && c(1) == op::DUP2
                && c(2) == op::ADD
                && c(3) == op::SWAP1
                && c(4) == op::DUP2
                && c(5) == op::LT
            {
                fused[cur] = op::ANDDUP2ADDSWAP1DUP2LT;
                for off in [1, 2, 3, 4, 5] {
                    fused[cur + off] = op::SNOP;
                }
                i += 6;
                continue;
            }
        }

        // ----------------------------
        // （1）AND SWAP1 POP SWAP2 SWAP1
        // （2）ISZERO PUSH2 _ _ JUMPI  ➜ JUMPIFZERO
        // （3）DUP2 MSTORE PUSH1 _ ADD ➜ DUP2MSTOREPUSH1ADD
        // ----------------------------
        if cur + 4 < fused.len() {
            let c = |o: usize| fused[cur + o];
            // (1)
            if c(0) == op::AND
                && c(1) == op::SWAP1
                && c(2) == op::POP
                && c(3) == op::SWAP2
                && c(4) == op::SWAP1
            {
                fused[cur] = op::ANDSWAP1POPSWAP2SWAP1;
                for off in [1, 2, 3, 4] {
                    fused[cur + off] = op::SNOP;
                }
                i += 5;
                continue;
            }
            // (2)
            if c(0) == op::ISZERO && c(1) == op::PUSH2 && c(4) == op::JUMPI {
                fused[cur] = op::JUMPIFZERO;
                for off in [1, 4] {
                    fused[cur + off] = op::SNOP;
                }
                i += 5;
                continue;
            }
            // (3)
            if c(0) == op::DUP2 && c(1) == op::MSTORE && c(2) == op::PUSH1 && c(4) == op::ADD {
                fused[cur] = op::DUP2MSTOREPUSH1ADD;
                for off in [1, 2, 4] {
                    fused[cur + off] = op::SNOP;
                }
                i += 5;
                continue;
            }
        }

        // ----------------------------
        // SWAP2 SWAP1 POP JUMP  ➜ SWAP2SWAP1POPJUMP
        // SWAP1 POP SWAP2 SWAP1 ➜ SWAP1POPSWAP2SWAP1
        // POP SWAP2 SWAP1 POP   ➜ POPSWAP2SWAP1POP
        // PUSH2 _ _ JUMP        ➜ PUSH2JUMP
        // PUSH2 _ _ JUMPI       ➜ PUSH2JUMPI
        // PUSH1 _ PUSH1         ➜ PUSH1PUSH1
        // ISZERO PUSH2 _ _      ➜ ISZEROPUSH2
        // ----------------------------
        if cur + 3 < fused.len() {
            let c = |o: usize| fused[cur + o];
            // (SWAP2 SWAP1 POP JUMP)
            if c(0) == op::SWAP2 && c(1) == op::SWAP1 && c(2) == op::POP && c(3) == op::JUMP {
                fused[cur] = op::SWAP2SWAP1POPJUMP;
                for off in [1, 2, 3] {
                    fused[cur + off] = op::SNOP;
                }
                i += 4;
                continue;
            }
            // (SWAP1 POP SWAP2 SWAP1)
            if c(0) == op::SWAP1 && c(1) == op::POP && c(2) == op::SWAP2 && c(3) == op::SWAP1 {
                fused[cur] = op::SWAP1POPSWAP2SWAP1;
                for off in [1, 2, 3] {
                    fused[cur + off] = op::SNOP;
                }
                i += 4;
                continue;
            }
            // (POP SWAP2 SWAP1 POP)
            if c(0) == op::POP && c(1) == op::SWAP2 && c(2) == op::SWAP1 && c(3) == op::POP {
                fused[cur] = op::POPSWAP2SWAP1POP;
                for off in [1, 2, 3] {
                    fused[cur + off] = op::SNOP;
                }
                i += 4;
                continue;
            }
            // (PUSH2 .. .. JUMP)
            if c(0) == op::PUSH2 && c(3) == op::JUMP {
                fused[cur] = op::PUSH2JUMP;
                fused[cur + 3] = op::SNOP;
                i += 4;
                continue;
            }
            // (PUSH2 .. .. JUMPI)
            if c(0) == op::PUSH2 && c(3) == op::JUMPI {
                fused[cur] = op::PUSH2JUMPI;
                fused[cur + 3] = op::SNOP;
                i += 4;
                continue;
            }
            // (PUSH1 _ PUSH1)
            if c(0) == op::PUSH1 && c(2) == op::PUSH1 {
                fused[cur] = op::PUSH1PUSH1;
                fused[cur + 2] = op::SNOP;
                i += 4;
                continue;
            }
            // (ISZERO PUSH2 .. ..)
            if c(0) == op::ISZERO && c(1) == op::PUSH2 {
                fused[cur] = op::ISZEROPUSH2;
                fused[cur + 1] = op::SNOP;
                i += 4;
                continue;
            }
        }

        // ----------------------------
        // PUSH1 _ ADD  ➜ PUSH1ADD
        // PUSH1 _ SHL  ➜ PUSH1SHL
        // PUSH1 _ DUP1 ➜ PUSH1DUP1
        // ----------------------------
        if cur + 2 < fused.len() {
            let inst0 = fused[cur];
            let inst2 = fused[cur + 2];
            if inst0 == op::PUSH1 {
                if inst2 == op::ADD {
                    fused[cur] = op::PUSH1ADD;
                    fused[cur + 2] = op::SNOP;
                    i += 3;
                    continue;
                }
                if inst2 == op::SHL {
                    fused[cur] = op::PUSH1SHL;
                    fused[cur + 2] = op::SNOP;
                    i += 3;
                    continue;
                }
                if inst2 == op::DUP1 {
                    fused[cur] = op::PUSH1DUP1;
                    fused[cur + 2] = op::SNOP;
                    i += 3;
                    continue;
                }
            }
        }

        // ----------------------------
        // SWAP1 POP        ➜ SWAP1POP
        // POP JUMP         ➜ POPJUMP
        // POP POP          ➜ POP2
        // SWAP2 SWAP1      ➜ SWAP2SWAP1
        // SWAP2 POP        ➜ SWAP2POP
        // DUP2 LT          ➜ DUP2LT
        // ----------------------------
        if cur + 1 < fused.len() {
            let inst0 = fused[cur];
            let inst1 = fused[cur + 1];
            if inst0 == op::SWAP1 && inst1 == op::POP {
                fused[cur] = op::SWAP1POP;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
            if inst0 == op::POP && inst1 == op::JUMP {
                fused[cur] = op::POPJUMP;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
            if inst0 == op::POP && inst1 == op::POP {
                fused[cur] = op::POP2;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
            if inst0 == op::SWAP2 && inst1 == op::SWAP1 {
                fused[cur] = op::SWAP2SWAP1;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
            if inst0 == op::SWAP2 && inst1 == op::POP {
                fused[cur] = op::SWAP2POP;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
            if inst0 == op::DUP2 && inst1 == op::LT {
                fused[cur] = op::DUP2LT;
                fused[cur + 1] = op::SNOP;
                i += 2;
                continue;
            }
        }

        if let Some(skip) = calculate_skip_steps(&fused, cur) {
            i += skip;
        }
        i += 1;
    }

    Ok(fused)
}

fn calculate_skip_steps(code: &[u8], cur: usize) -> Option<usize> {
    let inst = code[cur];

    if inst >= op::PUSH1 && inst <= op::PUSH32 {
        let steps = (inst - op::PUSH1 + 1) as usize;
        return Some(steps);
    }

    match inst {
        op::PUSH2JUMP | op::PUSH2JUMPI => Some(3), // (push2 imm16) + 1 (NOP)
        op::PUSH1PUSH1 => Some(3),                 // push1 imm1 + 1 (NOP)
        op::PUSH1ADD | op::PUSH1SHL | op::PUSH1DUP1 => Some(2),
        op::JUMPIFZERO => Some(4), // PUSH2 imm16 + NOP JUMPI replaced
        _ => None,
    }
}

/// BasicBlock represents a sequence of opcodes that can be executed linearly
/// without any jumps in or out except at the beginning and end.
#[derive(Debug)]
pub(crate) struct BasicBlock {
    /// inclusive start PC
    pub start_pc: usize,
    /// exclusive end PC
    pub end_pc: usize,
    /// raw bytes of opcodes in this block
    pub opcodes: Vec<u8>,
    // /// If this block ends with a jump, the target PC else None
    // pub jump_target: Option<usize>,
    /// Whether this block starts with a JUMPDEST
    pub is_jump_dest: bool,
}

impl BasicBlock {
    pub(crate) fn generate(code: &[u8]) -> Box<[Self]> {
        if code.is_empty() {
            return Vec::new().into_boxed_slice();
        }

        use std::collections::HashSet;

        // -------------- First pass: identify all JUMPDEST locations --------------
        let mut jump_dests: HashSet<usize> = HashSet::new();
        let mut pc = 0usize;
        while pc < code.len() {
            let op = code[pc];
            if op == op::JUMPDEST {
                jump_dests.insert(pc);
                pc += 1;
                continue
            }
            // Add 1 for the opcode byte
            if let Some(skip) = calculate_skip_steps(code, pc) {
                pc += 1 + skip;
            } else {
                pc += 1;
            }
        }

        // -------------- Second pass: build basic blocks --------------
        let mut blocks: Vec<BasicBlock> = Vec::new();
        pc = 0;
        let mut current: Option<BasicBlock> = None;
        while pc < code.len() {
            let op = code[pc];

            if op == op::INVALID || jump_dests.contains(&pc) {
                if let Some(mut blk) = current.take() {
                    blk.end_pc = pc;
                    blocks.push(blk);
                }
                current = Some(BasicBlock {
                    start_pc: pc,
                    end_pc: 0,
                    opcodes: Vec::new(),
                    // jump_target: None,
                    is_jump_dest: op == op::JUMPDEST,
                });
            } else if current.is_none() {
                current = Some(BasicBlock {
                    start_pc: pc,
                    end_pc: 0,
                    opcodes: Vec::new(),
                    // jump_target: None,
                    is_jump_dest: op == op::JUMPDEST,
                });
            }

            // Determine instruction length
            let (inst_len, _has_immediate) = if let Some(skip) = calculate_skip_steps(code, pc) {
                (1 + skip, true)
            } else {
                (1, false)
            };

            // Check bounds before accessing
            let bounded_len = if pc + inst_len > code.len() {
                code.len() - pc
            } else {
                inst_len
            };

            // Add instruction bytes to block
            if let Some(ref mut blk) = current {
                blk.opcodes.extend_from_slice(&code[pc..pc + bounded_len]);
            }

            pc += bounded_len;

            // If this is a block terminator (other than INVALID since we already handled it), end the block
            if is_block_terminator(op) {
                if let Some(mut blk) = current.take() {
                    blk.end_pc = pc;
                    // if (op == op::JUMP) && has_immediate {
                    //     let imm_start = blk.opcodes.len() - (inst_len - 1);
                    //     let imm_bytes = &blk.opcodes[imm_start..];

                    //     let mut tgt: usize = 0;
                    //     for &b in imm_bytes.iter().rev().take(8) {
                    //         tgt = (tgt << 8) | b as usize;
                    //     }
                    //     blk.jump_target = Some(tgt);
                    // }
                    blocks.push(blk);
                    current = None;
                }
            }
        }

        if let Some(mut blk) = current {
            blk.end_pc = pc;
            blocks.push(blk);
        }

        blocks.into_boxed_slice()
    }
}

fn is_block_terminator(op: u8) -> bool {
    matches!(
        op,
        op::STOP
            | op::RETURN
            | op::REVERT
            | op::SELFDESTRUCT
            | op::JUMP
            | op::JUMPI
    )
}

// =============================================================================
//  CFG-based opcode fusion – translated from provided Go implementation
// =============================================================================
pub(crate) fn do_basic_block_opcode_fusion(code: &[u8]) -> Result<Vec<u8>, FusionError> {
    // for byte in code {
    //     if *byte >= MIN_OPTIMIZED_OPCODE && *byte < MAX_OPTIMIZED_OPCODE {
    //         print!("{:?}", byte);
    //         return Err(FusionError::FailPreprocessing);
    //     }
    // }
    // if (MIN_OPTIMIZED_OPCODE..=MAX_OPTIMIZED_OPCODE).contains(&byte) {
    //     return Err(FusionError::FailPreprocessing);
    // }
    let blocks = BasicBlock::generate(code);
    if blocks.is_empty() {
        return Err(FusionError::FailPreprocessing);
    }

    let mut fused_code = code.to_vec();

    for (idx, block) in blocks.iter().enumerate() {
        let blk_ty = get_block_type(block, &blocks, idx);
        if matches!(blk_ty, BlockType::Others) {
            continue;
        }

        // print!("{:?} - {:?}\n", block.start_pc, block.end_pc);
        let mut pc = block.start_pc;
        while pc < block.end_pc && pc < code.len() {
            let byte = code[pc];
            if (MIN_OPTIMIZED_OPCODE..=MAX_OPTIMIZED_OPCODE).contains(&byte) {
                return Err(FusionError::FailPreprocessing);
            }
            if let Some(skip) = calculate_skip_steps(code, pc) {
                pc += 1 + skip;
            } else {
                pc += 1;
            }
        }

        let mut pc = block.start_pc;
        let mut has_invalid = false;
        while pc < block.end_pc && pc < code.len() {
            if code[pc] == op::INVALID {
                has_invalid = true;
                break;
            }
            if let Some(skip) = calculate_skip_steps(code, pc) {
                pc += 1 + skip;
            } else {
                pc += 1;
            }
        }
        if has_invalid {
            continue;
        }

        fuse_block(&mut fused_code, block)?;
    }

    Ok(fused_code)
}

// -----------------------------------------------------------------------------
//  Block-level helpers
// -----------------------------------------------------------------------------

#[derive(PartialEq, Eq)]
enum BlockType {
    Empty,
    EntryBB,
    JumpDest,
    ConditionalFallthrough,
    Others,
}

fn get_block_type(block: &BasicBlock, blocks: &[BasicBlock], index: usize) -> BlockType {
    if block.opcodes.is_empty() {
        return BlockType::Empty;
    }
    if block.start_pc == 0 {
        return BlockType::EntryBB;
    }
    if block.is_jump_dest {
        return BlockType::JumpDest;
    }
    if index > 0 {
        let prev = &blocks[index - 1];
        if let Some(&last) = prev.opcodes.last() {
            if last == op::JUMPI {
                return BlockType::ConditionalFallthrough;
            }
        }
    }
    BlockType::Others
}

fn fuse_block(code: &mut [u8], block: &BasicBlock) -> Result<(), FusionError> {
    let start = block.start_pc;
    let end = block.end_pc.min(code.len());
    if start >= end {
        return Ok(());
    }

    {
        let slice = &code[start..end];
        let fused_slice = do_code_fusion(slice)?;
        debug_assert_eq!(fused_slice.len(), slice.len());
        code[start..end].copy_from_slice(&fused_slice);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use bytecode::Bytecode;
    use std::fs;
    use primitives::{hex::{self, ToHexExt}, Bytes};
    use crate::opcode_optimizer::is_block_terminator;

    #[test]
    fn test_is_block_terminator() {
        assert_eq!(is_block_terminator(op::STOP), true);
        assert_eq!(is_block_terminator(op::RETURN), true);
        assert_eq!(is_block_terminator(op::REVERT), true);
        assert_eq!(is_block_terminator(op::SELFDESTRUCT), true);
        assert_eq!(is_block_terminator(op::JUMP), true);
        assert_eq!(is_block_terminator(op::JUMPI), true);

        assert_ne!(is_block_terminator(op::ADD), true);
        assert_ne!(is_block_terminator(op::GT), true);
        assert_ne!(is_block_terminator(op::ADDMOD), true);
        assert_ne!(is_block_terminator(op::LT), true);
        assert_ne!(is_block_terminator(op::PUSH1), true);
    }

    #[test]
    fn test_do_fusion() {
        let code = load_bytecode("/Users/wangtao/git_repo/revm_task/benchmark_test/bytecode/BIGA.bin");
        // println!("{:?}", Bytecode::new_raw(Bytes::from(code.clone())).legacy_jump_table().unwrap().as_slice());
        match do_basic_block_opcode_fusion(code.as_ref()) {
            // Ok(bytecode) => println!("{:?}", Bytecode::new_raw(Bytes::from(bytecode.clone())).legacy_jump_table().unwrap().as_slice()),
            Ok(bytecode) => println!("{:?}", bytecode.encode_hex()),
            Err(e) => panic!("{:?}", e),
        }

        match do_code_fusion(code.as_ref()) {
            Ok(bytecode) => println!("{:?}", bytecode.encode_hex()),
            Err(e) => panic!("{:?}", e),
        }
    }

    fn load_bytecode(path: &str) -> Bytes {
        let bytecode_str = fs::read_to_string(path)
            .expect("Failed to read bytecode file");
        let bytecode_str = bytecode_str.trim();
    
        if bytecode_str.starts_with("0x") {
            hex::decode(&bytecode_str[2..]).expect("Invalid hex in bytecode").into()
        } else {
            hex::decode(bytecode_str).expect("Invalid hex in bytecode").into()
        }
    }
}
