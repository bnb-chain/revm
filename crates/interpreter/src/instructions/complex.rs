use crate::{
    gas,
    interpreter_types::{InterpreterTypes, StackTr, Jumps, Immediates, MemoryTr, InputsTr},
    InstructionContext,
};
use primitives::{U256, B256};
use crate::InstructionResult;

use core::ptr;
use crate::interpreter_action::CallInput;

pub(super)fn and_swap1_pop_swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, 4*gas::VERYLOW+gas::BASE);

    popn!([a, b], context.interpreter);
    let r = a & b;
    backn!([c, d, e], context.interpreter);

    *e = *d;
    *d = *c;
    *c = r;

    context.interpreter.bytecode.relative_jump(4);
}


/// Fused instruction: SWAP2 SWAP1 POP JUMP
pub(super) fn swap2_swap1_pop_jump<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: SWAP2 + SWAP1 + POP + JUMP
    gas!(context.interpreter, 2*gas::VERYLOW + gas::BASE + gas::MID);

    // Pop two values: `a` will be re-inserted, `_` is discarded.
    popn!([a, _tmp], context.interpreter);

    // Read current top (will be the jump destination)
    backn!([top], context.interpreter);
    let dest_u256 = *top;
    // Replace top with `a`
    *top = a;

    // Validate jump destination
    let dest = as_usize_or_fail!(context.interpreter, dest_u256, InstructionResult::InvalidJump);
    if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    // Perform absolute jump
    context.interpreter.bytecode.absolute_jump(dest);
    // context.interpreter.bytecode.absolute_jump(dest);
}

/// Fused instruction: SWAP1 POP SWAP2 SWAP1
pub(super) fn swap1_pop_swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: SWAP1 + POP + SWAP2 + SWAP1
    gas!(context.interpreter, 3*gas::VERYLOW + gas::BASE);

    // if !context.interpreter.stack.exchange(0, 1) {
    //     context.interpreter.halt(InstructionResult::StackOverflow);
    // }

    // // Pop two (top is `a` to be re-inserted)
    // popn!([_tmp], context.interpreter);

    // if !context.interpreter.stack.exchange(0, 2) {
    //     context.interpreter.halt(InstructionResult::StackOverflow);
    // }

    // if !context.interpreter.stack.exchange(0, 1) {
    //     context.interpreter.halt(InstructionResult::StackOverflow);
    // }
    popn!([a], context.interpreter);
    backn!([b, c, d], context.interpreter);
    *d = *c;
    *c = *b;
    *b = a;

    // Skip over the remaining 3 bytes of the original sequence
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: POP SWAP2 SWAP1 POP
pub(super)fn pop_swap2_swap1_pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: POP + SWAP2 + SWAP1 + POP
    gas!(context.interpreter, 2*gas::BASE + 2*gas::VERYLOW);

    // Discard first value, keep `b`
    popn!([ _discard, b ], context.interpreter);
    backn!([d, c], context.interpreter);
    *c = *d;
    *d = b;

    // Skip remaining 3 bytes
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: PUSH2 <imm16> JUMP
pub(super)fn push2_jump<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // Gas: PUSH2 + JUMP
    gas!(context.interpreter, gas::VERYLOW + gas::MID);

    // Read immediate 2-byte destination (big-endian)
    let imm = context.interpreter.bytecode.read_slice(2);
    let dest = as_usize_or_fail!(context.interpreter, U256::from_be_slice(imm), InstructionResult::InvalidJump);

    if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    context.interpreter.bytecode.absolute_jump(dest);
}

/// Fused instruction: PUSH2 <imm16> JUMPI
pub(super)fn push2_jumpi<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // Gas: PUSH2 + JUMPI
    gas!(context.interpreter, gas::VERYLOW + gas::HIGH);

    // Read immediate destination
    let imm = context.interpreter.bytecode.read_slice(2);
    // Pop condition
    popn!([cond], context.interpreter);

    if !cond.is_zero() {
        let dest = as_usize_or_fail!(
            context.interpreter,
            U256::from_be_slice(imm),
            InstructionResult::InvalidJump
        );
        if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
            context.interpreter.halt(InstructionResult::InvalidJump);
            return;
        }
        context.interpreter.bytecode.absolute_jump(dest);
    } else {
        // Skip imm16 + NOP (total 3 bytes ahead of current pointer)
        context.interpreter.bytecode.relative_jump(3);
    }
}

/// Fused instruction: PUSH1 <a> PUSH1 <b>
pub(super)fn push1_push1<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // Gas: two PUSH1
    gas!(context.interpreter, 2*gas::VERYLOW);

    let bytes = context.interpreter.bytecode.read_slice(3);
    let a = U256::from(bytes.get(0).copied().unwrap_or(0u8));
    let b = U256::from(bytes.get(2).copied().unwrap_or(0u8));

    push!(context.interpreter, a);
    push!(context.interpreter, b);

    // Skip imm + NOP + imm (3 bytes)
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: PUSH1 <imm> ADD
pub(super)fn push1_add<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::VERYLOW);

    let imm = context.interpreter.bytecode.read_u8() as u64;
    backn!([b], context.interpreter);
    *b = *b + U256::from(imm);
    // push!(context.interpreter, res);

    // Skip imm + NOP (2 bytes)
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: PUSH1 <imm> SHL
pub(super)fn push1_shl<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::VERYLOW);

    let shift = context.interpreter.bytecode.read_u8();
    backn!([val], context.interpreter);

    *val = *val << (shift as usize);

    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: PUSH1 <imm> DUP1
pub(super)fn push1_dup1<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::VERYLOW);

    let imm = context.interpreter.bytecode.read_u8();
    let value = U256::from(imm);
    push!(context.interpreter, value);
    push!(context.interpreter, value);

    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: SWAP1 POP
pub(super)fn swap1_pop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::VERYLOW + gas::BASE);

    popn!([a], context.interpreter);
    let Some(b) = context.interpreter.stack.top() else {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    };
    *b = a;
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: POP JUMP
pub(super)fn pop_jump<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE + gas::MID);

    popn!([ _discard, dest_u256 ], context.interpreter);
    let dest = as_usize_or_fail!(context.interpreter, dest_u256, InstructionResult::InvalidJump);
    if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    context.interpreter.bytecode.absolute_jump(dest);
}

/// Fused instruction: POP POP
pub(super)fn pop2<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::BASE);
    popn!([ _a, _b ], context.interpreter);
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: SWAP2 SWAP1
pub(super)fn swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::VERYLOW);
    backn!([c, b, a], context.interpreter);
    let tmp = *a;
    *a = *b;
    *b = *c;
    *c = tmp;

    // if !context.interpreter.stack.exchange(0, 2) {
    //     context.interpreter.halt(InstructionResult::StackUnderflow);
    //     return;
    // }
    // if !context.interpreter.stack.exchange(0, 1) {
    //     context.interpreter.halt(InstructionResult::StackUnderflow);
    //     return;
    // }
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: SWAP2 POP
pub(super)fn swap2_pop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::VERYLOW + gas::BASE);

    // if !context.interpreter.stack.exchange(0, 2) {
    //     context.interpreter.halt(InstructionResult::StackUnderflow);
    //     return;
    // }
    backn!([c, _b, a], context.interpreter);
    *c = *a;
    // Pop the (now) top value
    popn!([ _x ], context.interpreter);
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: DUP2 LT
pub(super)fn dup2_lt<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2*gas::VERYLOW);

    // Duplicate 2nd item to top then perform LT
    // if !context.interpreter.stack.dup(2) {
    //     context.interpreter.halt(InstructionResult::StackUnderflow);
    //     return;
    // }

    // Pop the two operands
    // popn!([a, b], context.interpreter);
    backn!([b, a], context.interpreter);
    *a = if *b < *a { U256::ONE } else { U256::ZERO };
    // push!(context.interpreter, result);

    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: ISZERO PUSH2 .. JUMPI  => JUMPIFZERO
pub(super)fn jump_if_zero<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Approximated gas: ISZERO (VERYLOW) + JUMPI (HIGH)
    gas!(context.interpreter, 2*gas::VERYLOW + gas::HIGH);

    // Pop condition value
    popn!([value], context.interpreter);

    if value.is_zero() {
        // Immediate destination is 2 bytes located 2 bytes ahead (skip NOP + imm16)
        let dest = context.interpreter.bytecode.read_offset_u16(1) as usize;
        // let dest = dest_u16 as usize;
        if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
            context.interpreter.halt(InstructionResult::InvalidJump);
            return;
        }
        // println!("{:?}, {:?}", dest, context.interpreter.stack.len());
        // println!("{:?}, {:?}", context.interpreter.bytecode.read_offset_u16(3), context.interpreter.bytecode.read_offset_u16(5));
        context.interpreter.bytecode.absolute_jump(dest);
    } else {
        // Skip the rest (NOP + imm16 + NOP) => 4 bytes
        context.interpreter.bytecode.relative_jump(4);
    }
}

/// Super NOP instruction (SNOP)
pub(super)fn snop<WIRE: InterpreterTypes, H: ?Sized>(_context: InstructionContext<'_, H, WIRE>) {
    // Zero-cost, zero-effect.
    // gas!(context.interpreter, gas::ZERO);
    // Nothing else to do.
}


/// Fused instruction: ISZERO PUSH2 <imm16>
pub(super)fn iszero_push2<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, 2 * gas::VERYLOW);

    // Mutate top of stack
    let Some(x) = context.interpreter.stack.top() else {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    };
    if x.is_zero() {
        *x = U256::ONE;
    } else {
        *x = U256::ZERO;
    }
    let _ = x;

    // Current PC is at NOP (byte after fused opcode)
    // Immediate bytes are located at offset 1 and 2
    let imm16 = context.interpreter.bytecode.read_offset_u16(1);
    push!(context.interpreter, U256::from(imm16));

    // Skip NOP + imm16 (3 bytes total)
    context.interpreter.bytecode.relative_jump(3);
}


/// Fused instruction: DUP2 MSTORE PUSH1 _ ADD
pub(super)fn dup2_mstore_push1_add<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: MSTORE (VERYLOW) + PUSH1 + ADD + small overhead
    gas!(context.interpreter, 4*gas::VERYLOW);

    // Pop value to store
    popn!([val], context.interpreter);

    // Obtain offset (now at stack top)
    let Some(offset_ref) = context.interpreter.stack.top() else {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    };
    let offset_usize = as_usize_or_fail!(context.interpreter, *offset_ref);
    // Resize memory and store 32-byte word
    resize_memory!(context.interpreter, offset_usize, 32);
    context
        .interpreter
        .memory
        .set(offset_usize, &val.to_be_bytes::<32>());

    // Read immediate byte (located +2 from current ptr: NOP + imm)
    let imm = context.interpreter.bytecode.read_slice(3)[2];
    *offset_ref = *offset_ref + U256::from(imm);
    // *offset_ref = res;

    // Skip remaining bytes (NOP, imm, NOP) => 4 bytes
    context.interpreter.bytecode.relative_jump(4);
}

/// Fused instruction: DUP1 PUSH4 imm EQ PUSH2 imm2
pub(super)fn dup1_push4_eq_push2<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 4*gas::VERYLOW + gas::BASE);
    gas!(context.interpreter, 4*gas::VERYLOW);

    // Duplicate top
    if !context.interpreter.stack.dup(1) {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    }
    context.interpreter.bytecode.relative_jump(1);

    // Read 4-byte constant (offsets 2-5 from current ptr)
    let bytes4 = context.interpreter.bytecode.read_slice(4);
    let const_val = U256::from_be_slice(&bytes4);
    backn!([x], context.interpreter);
    *x = if const_val == *x {U256::ONE} else { U256::ZERO }; 
    // push!(context.interpreter, const_val);
    context.interpreter.bytecode.relative_jump(6);

    // // Equality check
    // popn!([a, x], context.interpreter); // a = const_val, x = duplicated original
    // let eq = if a == x { U256::ONE } else { U256::ZERO };
    // push!(context.interpreter, eq);

    // Read 2-byte immediate for PUSH2 (offset 8-9 from current ptr)
    let dest_u16 = context.interpreter.bytecode.read_u16();
    push!(context.interpreter, U256::from(dest_u16));

    // Skip remaining bytes to end of fused sequence (total 10 ⇒ skip 9)
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: PUSH1 CALLDATALOAD PUSH1 SHR DUP1 PUSH4 GT PUSH2
pub(super)fn push1_calldataload_push1_shr_dup1_push4_gt_push2<
    WIRE: InterpreterTypes,
    H: ?Sized,
>(context: InstructionContext<'_, H, WIRE>) {
    // Rough gas: PUSH1+CALLDATALOAD+PUSH1+SHR+DUP1+PUSH4+GT+PUSH2
    // gas!(
    //     context.interpreter,
    //     4 * gas::VERYLOW + gas::MID + gas::HIGH + gas::BASE
    // );
    gas!(context.interpreter, 8*gas::VERYLOW);

    // Read immediate offset (1 byte right after opcode)
    let bytes = context.interpreter.bytecode.read_slice(15);
    if bytes.len() < 15 {
        context.interpreter.halt(InstructionResult::InvalidOperandOOG);
        return;
    }
    let offset_byte = bytes[0] as usize;

    // Load 32 bytes from calldata at offset
    let mut word = B256::ZERO;
    let input = context.interpreter.input.input();
    let input_len = input.len();
    if offset_byte < input_len {
        let count = 32.min(input_len - offset_byte);
        match input {
            CallInput::Bytes(bytes) => unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr().add(offset_byte), word.as_mut_ptr(), count);
            },
            CallInput::SharedBuffer(range) => {
                let slice = context.interpreter.memory.global_slice(range.clone());
                unsafe {
                    ptr::copy_nonoverlapping(slice.as_ptr().add(offset_byte), word.as_mut_ptr(), count);
                }
            }
        }
    }
    // let mut x = U256::from_be_bytes();

    // Read shift immediate (byte 4 in slice: index 3 is PUSH1 opcode (NOP), index 4 is imm)
    let shift_byte = bytes[3] as usize;
    let mut x = word.into();
    x = x >> shift_byte;

    // Constant 4-byte big-endian located starting at index 8..12
    let const_val = U256::from_be_slice(&bytes[7..11]);
    // println!("{:?}", const_val);

    // // Push x
    push!(context.interpreter, x);
    // push!(context.interpreter, x);
    

    // GT: compare const_val (a) and copy of x (p)
    // popn_top!([_a], p_ref, context.interpreter);
    if const_val > x {
        push!(context.interpreter, U256::ONE);
    } else {
        push!(context.interpreter, U256::ZERO);
    }

    // PUSH2 dest (index 14,15)
    let dest_u16 = ((bytes[13] as u16) << 8) | bytes[14] as u16;
    push!(context.interpreter, U256::from(dest_u16));

    // Skip remaining 15 bytes (pattern length 16)
    context.interpreter.bytecode.relative_jump(15);
}

/// Fused instruction: PUSH1 PUSH1 PUSH1 SHL SUB
pub(super)fn push1_push1_push1_shl_sub<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2 * gas::VERYLOW + gas::BASE);
    gas!(context.interpreter, 5*gas::VERYLOW);

    // Read the three immediates
    let bytes = context.interpreter.bytecode.read_slice(7);
    if bytes.len() < 7 {
        context.interpreter.halt(InstructionResult::InvalidOperandOOG);
        return;
    }
    let imm1 = U256::from(bytes[0]); // first PUSH1 immediate
    let imm2 = U256::from(bytes[2]); // second
    let imm3 = bytes[4] as usize; // third is shift amount

    let mut res = imm2 << imm3;
    res = res.wrapping_sub(imm1);

    push!(context.interpreter, res);

    // Skip remaining 7 bytes
    context.interpreter.bytecode.relative_jump(7);
}

/// Fused instruction: SWAP1 PUSH1 DUP1 NOT SWAP2 ADD AND DUP2 ADD SWAP1 DUP2 LT
pub(super)fn swap1_push1_dup1_not_swap2_add_and_dup2_add_swap1_dup2_lt<
    WIRE: InterpreterTypes,
    H: ?Sized,
>(context: InstructionContext<'_, H, WIRE>) {
    // gas!(context.interpreter, 10 * gas::VERYLOW);
    gas!(context.interpreter, 12*gas::VERYLOW);

    // 1. SWAP1
    // if !context.interpreter.stack.exchange(0, 1) {
    //     context.interpreter.halt(InstructionResult::StackUnderflow);
    //     return;
    // }

    backn!([b,a], context.interpreter);

    // 2. PUSH1 immediate (byte index 2)
    context.interpreter.bytecode.relative_jump(1);
    let imm = context.interpreter.bytecode.read_u8();
    // println!("{:?}", imm);
    // push!(context.interpreter, U256::from(imm));
    // b, a, im, ~im
    // a, ~im & im+b, a
    let imm = U256::from(imm);
    let tmp = (!imm & (imm+*b)) + *a;
    // println!("{:?}", tmp);
    // *a = *b;

    if tmp < *b {
        *a = U256::ONE;
    } else {
        *a = U256::ZERO;
    }
    *b = tmp;

    // Skip remaining 12 bytes (pattern length 13)
    context.interpreter.bytecode.relative_jump(11);
}

/// Fused instruction: AND DUP2 ADD SWAP1 DUP2 LT
pub(super)fn and_dup2_add_swap1_dup2_lt<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 5 * gas::VERYLOW);
    gas!(context.interpreter, 6*gas::VERYLOW);

    // Step 1: AND (pop x, y; push y&x)
    popn!([a], context.interpreter);
    backn!([c, b], context.interpreter);
    let tmp = *c;
    *c = a+*b+*c;
    if *c < tmp {
        *b = U256::ONE;
    } else {
        *b = U256::ZERO;
    }

    // Skip remaining 5 bytes (pattern length 6)
    context.interpreter.bytecode.relative_jump(5);
}


#[cfg(test)]
mod fused_tests {
    use super::*;
    use crate::{interpreter, InstructionContext};
    use crate::interpreter::{Interpreter, EthInterpreter, ExtBytecode};
    use crate::instructions::{bitwise, stack, control, arithmetic, memory, system};
    use bitvec::{bitvec, order::Lsb0, vec::BitVec};
    use bytecode::{Bytecode, JumpTable};
    use primitives::Bytes;

    type Interp = Interpreter<EthInterpreter>;

    // helper
    fn make_interp(len: usize) -> Interp {
        let mut i = Interp::default_ext();
        let dummy = Bytecode::new_legacy(Bytes::from(vec![0u8; len]));
        i.bytecode = ExtBytecode::new(dummy);
        i
    }

    fn make_interp_with_jump(len: usize, jump_loc: usize) -> Interp {
        let mut i = Interp::default_ext();
        let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; len];
        unsafe {jumps.set_unchecked(jump_loc, true) }
        let mut v = vec![0u8; len];
        for i in 0..len-1 {
            v[i] = i as u8;
        }

        let dummy = Bytecode::new_analyzed(Bytes::from(v), len, JumpTable::new(jumps));
        i.bytecode = ExtBytecode::new(dummy);
        i
    }

    fn run<F>(mut interp: Interp, f: F) -> (Interp, usize)
    where F: FnOnce(&mut Interp) {
        f(&mut interp);
        let pc = interp.bytecode.pc();
        (interp, pc)
    }

    #[test]
    fn test_and_swap1_pop_swap2_swap1() {
        let (mut interp, pc) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            and_swap1_pop_swap2_swap1(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, _) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            bitwise::bitand(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::pop(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 4);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_swap1_pop_swap2_swap1() {
        let (interp, pc) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            swap1_pop_swap2_swap1(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, _) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::pop(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 3);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_swap2_swap1_pop_jump() {
        let (interp, pc) = run(make_interp_with_jump(10, 7), |ip| {
            // stack: [dest, keep, discard, extra...]
            let _ = ip.stack.push(U256::from(7)); // jump dest
            let _ = ip.stack.push(U256::from(0xaa));
            let _ = ip.stack.push(U256::from(0xbb));
            // println!("{:?}", ip.stack);
            swap2_swap1_pop_jump(InstructionContext{ host: &mut (), interpreter: ip });
            // println!("{:?}, {:?}", ip.stack, ip.bytecode.pc());
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 7), |ip| {
            // stack: [dest, keep, discard, extra...]
            let _ = ip.stack.push(U256::from(7)); // jump dest
            let _ = ip.stack.push(U256::from(0xaa));
            let _ = ip.stack.push(U256::from(0xbb));
            stack::swap::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::pop(InstructionContext{ host: &mut (), interpreter: ip });
            control::jump(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 7);
        assert_eq!(pc2, 7);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_pop_swap2_swap1_pop() {
        let (interp, pc) = run(make_interp(10), |ip| {
            for n in 0..4 {
                let _ = ip.stack.push(U256::from(n));
            }
            pop_swap2_swap1_pop(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, _pc2) = run(make_interp(10), |ip| {
            for n in 0..4 {
                let _ = ip.stack.push(U256::from(n));
            }
            stack::pop(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            stack::pop(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 3);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_swap2_pop() {
        let (interp, pc) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            swap2_pop(InstructionContext { host: &mut (), interpreter: ip });
        });

        let (interp2, pc2) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            stack::swap::<2, _, _>(InstructionContext { host: &mut (), interpreter: ip });
            stack::pop(InstructionContext { host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
        });

        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push2_jump() {
        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            push2_jump(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            stack::push::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            control::jump(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push2_jumpi() {
        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(5));
            push2_jumpi(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(5));
            stack::push::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            control::jumpi(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);

        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(0));
            push2_jumpi(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(0));
            // println!("{:?}", ip.bytecode.pc());
            stack::push::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            control::jumpi(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, 3);
        assert_eq!(pc2, 3);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push1_add() {
        let (mut interp, pc) = run(make_interp(4), |ip| {
            let _ = ip.stack.push(U256::ONE);
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![5,0,0,0].into()));
            push1_add(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(6));
        assert_eq!(pc, 2);
    }

    #[test]
    fn test_pop2() {
        let (interp, pc) = run(make_interp(2), |ip| {
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            pop2(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert!(interp.stack.is_empty());
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_dup2_lt() {
        let (mut interp, pc) = run(make_interp(2), |ip| {
            let _ = ip.stack.push(U256::from(3));
            let _ = ip.stack.push(U256::from(4));
            dup2_lt(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::ONE); // 4 < 3 false, but after swap → comparison 3<4 ⇒ true
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_push1_shl() {
        let (mut interp, pc) = run(make_interp_with_jump(3, 0), |ip| {
            ip.bytecode.relative_jump(1);
            let _ = ip.stack.push(U256::from(4));
            push1_shl(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(8));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_push1_dup1() {
        let (mut interp, pc) = run(make_interp_with_jump(3, 0), |ip| {
            ip.bytecode.relative_jump(1);
            push1_dup1(InstructionContext{ host: &mut (), interpreter: ip });
            // stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            // ip.bytecode.relative_jump(1);
            // stack::dup::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(1));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_swap1_pop() {
        let (mut interp, pc) = run(make_interp_with_jump(3, 0), |ip| {
            ip.bytecode.relative_jump(1);
            let _ = ip.stack.push(U256::from(3));
            let _ = ip.stack.push(U256::from(4));
            swap1_pop(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(4));
        assert_eq!(pc, 2);
    }

    #[test]
    fn test_pop_jump() {
        let (mut interp, pc) = run(make_interp_with_jump(10, 3), |ip| {
            ip.bytecode.relative_jump(1);
            let _ = ip.stack.push(U256::from(2));
            let _ = ip.stack.push(U256::from(3));
            let _ = ip.stack.push(U256::from(4));
            pop_jump(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(2));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_jump_if_zero() {
        let (interp_true, pc_true) = run(make_interp_with_jump(260, 258), |ip| {
            let _ = ip.stack.push(U256::ZERO);
            jump_if_zero(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc_true, 258);

        let (_interp_false, pc_false) = run(make_interp_with_jump(10, 0), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_legacy(Bytes::from(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0])));
            let _ = ip.stack.push(U256::ONE);
            jump_if_zero(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc_false, 4);
    }

    #[test]
    fn test_push1_push1() {
        let (mut interp, pc) = run(make_interp(5), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![0x05, 0x00, 0x07, 0, 0].into()));
            push1_push1(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let mut stack_vals = interp.stack.data().clone();
        assert_eq!(stack_vals.pop(), Some(U256::from(7u8)));
        assert_eq!(stack_vals.pop(), Some(U256::from(5u8)));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_iszero_push2() {
        let (mut interp, pc) = run(make_interp(5), |ip| {
            // bytecode: NOP, imm_hi, imm_lo, ...
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![0, 0x12, 0x34, 0, 0].into()));
            let _ = ip.stack.push(U256::ZERO);
            iszero_push2(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::from(0x1234u16));
        let second = interp.stack.data().get(interp.stack.len()-2).unwrap();
        assert_eq!(*second, U256::ONE);
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_push1_push1_push1_shl_sub() {
        let (mut interp, pc) = run(make_interp(10), |ip| {
            // immediates: imm1=2, imm2=3, imm3=1 => (3<<1)-2 = 4
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![2, 0, 3, 0, 1, 0, 0, 0, 0, 0].into()));
            push1_push1_push1_shl_sub(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            // immediates: imm1=2, imm2=3, imm3=1 => (3<<1)-2 = 4
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![2, 0, 3, 0, 1, 0, 0, 0, 0, 0].into()));
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::shl(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            arithmetic::sub(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(4u8));
        assert_eq!(pc, 7);
        assert_eq!(interp2.stack.top().unwrap(), &U256::from(4u8));
        assert_eq!(pc2, 7);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_and_dup2_add_swap1_dup2_lt() {
        let (mut interp, pc) = run(make_interp(10), |ip| {
            let _ = ip.stack.push(U256::from(3)); // c (bottom of 3 values)
            let _ = ip.stack.push(U256::from(2)); // b
            let _ = ip.stack.push(U256::from(1)); // a (top)
            and_dup2_add_swap1_dup2_lt(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            let _ = ip.stack.push(U256::from(3)); // c (bottom of 3 values)
            let _ = ip.stack.push(U256::from(2)); // b
            let _ = ip.stack.push(U256::from(1)); // a (top)
            arithmetic::add(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::lt(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::ZERO);
        let second = interp.stack.data().get(interp.stack.len()-2).unwrap();
        assert_eq!(*second, U256::from(6u8));
        assert_eq!(pc, 5);
        assert_eq!(pc2, 5);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_dup2_mstore_push1_add() {
        // initial stack: offset(0x20), val(0x0a)
        let (mut interp, pc) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![0, 0, 7, 0, 0, 0, 0, 0, 0, 0].into()));
            let _ = ip.stack.push(U256::from(0x20u64));
            let _ = ip.stack.push(U256::from(0x0au8));
            dup2_mstore_push1_add(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![0, 0, 7, 0, 0, 0, 0, 0, 0, 0].into()));
            let _ = ip.stack.push(U256::from(0x20u64));
            let _ = ip.stack.push(U256::from(0x0au8));
            stack::dup::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            memory::mstore(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext{ host: &mut (), interpreter: ip });
        });
        // offset should be 0x20+7=0x27 at top
        assert_eq!(interp.stack.top().unwrap(), &U256::from(0x27u64));
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_dup1_push4_eq_push2() {
        // const value 0x01020304, dest=0x0003
        let bytes = vec![0, 0x01,0x02,0x03,0x04, 0, 0x00, 0x03, 0];
        let (mut interp, pc) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            let _ = ip.stack.push(U256::from_be_slice(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,0,0x01,0x02,0x03,0x04]));
            dup1_push4_eq_push2(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            let _ = ip.stack.push(U256::from_be_slice(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0, 0,0,0,0x01,0x02,0x03,0x04]));
            stack::dup::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<4, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::eq(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
        });
        // top dest
        assert_eq!(interp.stack.top().unwrap(), &U256::from(768u16));
        // eq result should be ONE
        let second = interp.stack.data().get(interp.stack.len()-2).unwrap();
        assert_eq!(*second, U256::ONE);
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }
 
    #[test]
    fn test_swap1_push1_dup1_not_swap2_add_and_dup2_add_swap1_dup2_lt() {
        let imm: u8 = 1; // simple
        // prepare dummy bytecode length 13 (pattern length) with imm at index 1
        let mut bytes = vec![0u8; 13];
        bytes[1] = imm;
        let (mut interp, pc) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // initial stack top a=1, b=2 (note reversed order top-first)
            let _ = ip.stack.push(U256::from(2)); // b (top)
            let _ = ip.stack.push(U256::from(1)); // a (second)
            swap1_push1_dup1_not_swap2_add_and_dup2_add_swap1_dup2_lt(InstructionContext{ host: &mut (), interpreter: ip });
        });
        let (mut interp2, pc2) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // initial stack top a=1, b=2 (note reversed order top-first)
            let _ = ip.stack.push(U256::from(2)); // b (top)
            let _ = ip.stack.push(U256::from(1)); // a (second)
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::not(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::swap::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::bitand(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::lt(InstructionContext{ host: &mut (), interpreter: ip });
        });
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }

    use primitives::hex::decode;
    #[test]
    fn test_push1_calldataload_push1_shr_dup1_push4_gt_push2() {
        // Setup immediate values: offset=0, shift=0, const=0x00000002 (> x), dest=0x0004
        let mut bytes = vec![0u8; 16];
        let input = decode("70a082310000000000000000000000001000000000000000000000000000000000000001").unwrap();
        bytes[0] = 0; // offset
        bytes[3] = 0xe0; // shift
        bytes[7] = 0x89; bytes[8] = 0x3d; bytes[9]=0x20; bytes[10]=0xe8; // const
        bytes[13]=0; bytes[14]=0xad; // dest =4

        let (mut interp, pc) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // Provide calldata: 32 bytes zero so x=0
            ip.input.input = CallInput::Bytes(Bytes::from(input.clone()));
            push1_calldataload_push1_shr_dup1_push4_gt_push2(InstructionContext{ host: &mut (), interpreter: ip });
        });

        let (mut interp2, pc2) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // Provide calldata: 32 bytes zero so x=0
            ip.input.input = CallInput::Bytes(Bytes::from(input));
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            system::calldataload(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::shr(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::dup::<1, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<4, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            bitwise::gt(InstructionContext{ host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::push::<2, _, _>(InstructionContext{ host: &mut (), interpreter: ip });
        });

        // Stack top dest (0x0004)
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::from(173u16));
        // second should be ONE (const > x)
        let second = interp.stack.data().get(interp.stack.len()-2).unwrap();
        assert_eq!(*second, U256::ONE);
        assert_eq!(pc, 15);
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_swap2_swap1() {
        let (mut interp, pc) = run(make_interp(5), |ip| {
            // bottom 0,1,2 top
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            swap2_swap1(InstructionContext { host: &mut (), interpreter: ip });
        });

        let (mut interp2, pc2) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            stack::swap::<2, _, _>(InstructionContext { host: &mut (), interpreter: ip });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext { host: &mut (), interpreter: ip });
        });

        assert_eq!(pc, pc2);
        assert_eq!(pc, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_snop() {
        let (mut interp, pc) = run(make_interp(1), |ip| {
            let _ = ip.stack.push(U256::from(123));
            snop(InstructionContext { host: &mut (), interpreter: ip });
        });

        assert_eq!(pc, 0);
        assert_eq!(interp.stack.top().unwrap(), &U256::from(123u64));
    }

}
