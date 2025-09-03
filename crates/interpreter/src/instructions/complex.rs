use crate::InstructionResult;
use crate::{
    gas,
    instructions::i256::i256_cmp,
    interpreter_types::{Immediates, InputsTr, InterpreterTypes, Jumps, MemoryTr, StackTr},
    InstructionContext,
};
use primitives::{B256, U256};

use crate::interpreter_action::CallInput;
use core::ptr;

// ============================ Super-Instructions ============================

pub(super) fn and_swap1_pop_swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 4*gas::VERYLOW+gas::BASE);

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
    // gas!(context.interpreter, 2*gas::VERYLOW + gas::BASE + gas::MID);

    // Pop two values: `a` will be re-inserted, `_` is discarded.
    popn!([a, _tmp], context.interpreter);

    // Read current top (will be the jump destination)
    backn!([top], context.interpreter);
    let dest_u256 = *top;
    // Replace top with `a`
    *top = a;

    // Validate jump destination
    let dest = as_usize_or_fail!(
        context.interpreter,
        dest_u256,
        InstructionResult::InvalidJump
    );
    if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    // Perform absolute jump
    context.interpreter.bytecode.absolute_jump(dest);
}

/// Fused instruction: SWAP1 POP SWAP2 SWAP1
pub(super) fn swap1_pop_swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: SWAP1 + POP + SWAP2 + SWAP1
    // gas!(context.interpreter, 3*gas::VERYLOW + gas::BASE);

    popn!([a], context.interpreter);
    backn!([b, c, d], context.interpreter);
    *d = *c;
    *c = *b;
    *b = a;

    // Skip over the remaining 3 bytes of the original sequence
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: POP SWAP2 SWAP1 POP
pub(super) fn pop_swap2_swap1_pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: POP + SWAP2 + SWAP1 + POP
    gas!(context.interpreter, 2 * gas::BASE + 2 * gas::VERYLOW);

    // Discard first value, keep `b`
    popn!([_discard, b], context.interpreter);
    backn!([d, c], context.interpreter);
    *c = *d;
    *d = b;

    // Skip remaining 3 bytes
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: PUSH2 <imm16> JUMP
pub(super) fn push2_jump<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: PUSH2 + JUMP
    // gas!(context.interpreter, gas::VERYLOW + gas::MID);

    // Read immediate 2-byte destination (big-endian)
    let imm = context.interpreter.bytecode.read_slice(2);
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
}

/// Fused instruction: PUSH2 <imm16> JUMPI
pub(super) fn push2_jumpi<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: PUSH2 + JUMPI
    // gas!(context.interpreter, gas::VERYLOW + gas::HIGH);

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
pub(super) fn push1_push1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: two PUSH1
    // gas!(context.interpreter, 2*gas::VERYLOW);

    let bytes = context.interpreter.bytecode.read_slice(3);
    let a = U256::from(bytes.get(0).copied().unwrap_or(0u8));
    let b = U256::from(bytes.get(2).copied().unwrap_or(0u8));

    push!(context.interpreter, a);
    push!(context.interpreter, b);

    // Skip imm + NOP + imm (3 bytes)
    context.interpreter.bytecode.relative_jump(3);
}

/// Fused instruction: PUSH1 <imm> ADD
pub(super) fn push1_add<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2*gas::VERYLOW);

    let imm = context.interpreter.bytecode.read_u8() as u64;
    backn!([b], context.interpreter);
    *b = *b + U256::from(imm);
    // push!(context.interpreter, res);

    // Skip imm + NOP (2 bytes)
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: PUSH1 <imm> SHL
pub(super) fn push1_shl<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2*gas::VERYLOW);

    let shift = context.interpreter.bytecode.read_u8();
    backn!([val], context.interpreter);

    *val = *val << (shift as usize);

    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: PUSH1 <imm> DUP1
pub(super) fn push1_dup1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2*gas::VERYLOW);

    let imm = context.interpreter.bytecode.read_u8();
    let value = U256::from(imm);
    push!(context.interpreter, value);
    push!(context.interpreter, value);

    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: SWAP1 POP
pub(super) fn swap1_pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, gas::VERYLOW + gas::BASE);

    popn!([a], context.interpreter);
    let Some(b) = context.interpreter.stack.top() else {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    };
    *b = a;
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: POP JUMP
pub(super) fn pop_jump<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, gas::BASE + gas::MID);

    popn!([_discard, dest_u256], context.interpreter);
    let dest = as_usize_or_fail!(
        context.interpreter,
        dest_u256,
        InstructionResult::InvalidJump
    );
    if !context.interpreter.bytecode.is_valid_legacy_jump(dest) {
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    context.interpreter.bytecode.absolute_jump(dest);
}

/// Fused instruction: POP POP
pub(super) fn pop2<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // gas!(context.interpreter, 2*gas::BASE);
    popn!([_a, _b], context.interpreter);
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: SWAP2 SWAP1
pub(super) fn swap2_swap1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2*gas::VERYLOW);
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
pub(super) fn swap2_pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, gas::VERYLOW + gas::BASE);

    backn!([c, _b, a], context.interpreter);
    *c = *a;
    // Pop the (now) top value
    popn!([_x], context.interpreter);
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: DUP2 LT
pub(super) fn dup2_lt<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    // gas!(context.interpreter, 2*gas::VERYLOW);

    backn!([b, a], context.interpreter);
    *a = if *b < *a { U256::ONE } else { U256::ZERO };
    // push!(context.interpreter, result);

    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: ISZERO PUSH2 .. JUMPI  => JUMPIFZERO
pub(super) fn jump_if_zero<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Approximated gas: ISZERO (VERYLOW) + JUMPI (HIGH)
    // gas!(context.interpreter, 2*gas::VERYLOW + gas::HIGH);

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
pub(super) fn snop<WIRE: InterpreterTypes, H: ?Sized>(_context: InstructionContext<'_, H, WIRE>) {
    // Zero-cost, zero-effect.
    // gas!(context.interpreter, gas::ZERO);
    // Nothing else to do.
}

/// Fused instruction: ISZERO PUSH2 <imm16>
pub(super) fn iszero_push2<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2 * gas::VERYLOW);

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
pub(super) fn dup2_mstore_push1_add<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // Gas: MSTORE (VERYLOW) + PUSH1 + ADD + small overhead
    // gas!(context.interpreter, 4*gas::VERYLOW);

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
pub(super) fn dup1_push4_eq_push2<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 4*gas::VERYLOW);

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
    *x = if const_val == *x {
        U256::ONE
    } else {
        U256::ZERO
    };
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
pub(super) fn push1_calldataload_push1_shr_dup1_push4_gt_push2<
    WIRE: InterpreterTypes,
    H: ?Sized,
>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 8*gas::VERYLOW);

    // Read immediate offset (1 byte right after opcode)
    let bytes = context.interpreter.bytecode.read_slice(15);
    if bytes.len() < 15 {
        context
            .interpreter
            .halt(InstructionResult::InvalidOperandOOG);
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
                    ptr::copy_nonoverlapping(
                        slice.as_ptr().add(offset_byte),
                        word.as_mut_ptr(),
                        count,
                    );
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
pub(super) fn push1_push1_push1_shl_sub<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 5*gas::VERYLOW);

    // Read the three immediates
    let bytes = context.interpreter.bytecode.read_slice(7);
    if bytes.len() < 7 {
        context
            .interpreter
            .halt(InstructionResult::InvalidOperandOOG);
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
pub(super) fn swap1_push1_dup1_not_swap2_add_and_dup2_add_swap1_dup2_lt<
    WIRE: InterpreterTypes,
    H: ?Sized,
>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 12*gas::VERYLOW);

    backn!([b, a], context.interpreter);

    // 2. PUSH1 immediate (byte index 2)
    context.interpreter.bytecode.relative_jump(1);
    let imm = context.interpreter.bytecode.read_u8();

    let imm = U256::from(imm);
    let tmp = (!imm & (imm + *b)) + *a;

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
pub(super) fn and_dup2_add_swap1_dup2_lt<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 6*gas::VERYLOW);

    // Step 1: AND (pop x, y; push y&x)
    popn!([a], context.interpreter);
    backn!([c, b], context.interpreter);
    let tmp = *c;
    *c = a + *b + *c;
    if *c < tmp {
        *b = U256::ONE;
    } else {
        *b = U256::ZERO;
    }

    // Skip remaining 5 bytes (pattern length 6)
    context.interpreter.bytecode.relative_jump(5);
}

// ============================ New Fused Instructions from Go Example ============================

/// Fused instruction: DUP3 AND
/// Takes the 3rd element from stack top, performs AND with top element
/// x := scope.Stack.data[scope.Stack.len()-3], y := scope.Stack.peek(), y.And(&x, y)
pub(super) fn dup3_and<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, gas::VERYLOW * 2);

    backn!([b3, _b2, b1], context.interpreter);
    *b1 = *b1 & *b3;
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: SWAP2 SWAP1 DUP3 SUB SWAP2 DUP3 GT PUSH2
/// This is a complex sequence that performs stack manipulation and comparison
//  Initial stack: [c, d, e] (top = c, 2nd = d, 3rd = e)
//
// #	Opcode	What it does (top-first)	                                Before	                                 After
// 1	SWAP2	Swap top and 3rd items  (a,b,c → c,b,a)	                    [c, d, e]                                [e, d, c]
// 2	SWAP1	Swap top and 2nd items (a,b → b,a)	                        [e, d, c]	                             [d, e, c]
// 3	DUP3	Duplicate the 3rd item (from top) and push it on top	    [d, e, c]                    	         [c, d, e, c]
// 4	SUB	Pop a (top) and b (next), push a − b (wraps mod 2²⁵⁶)	        [c, d, e, c]	                         [c − d, e, c]
// 5	SWAP2	Swap top and 3rd items	                                    [c − d, e, c]	                         [c, e, c − d]
// 6	DUP3	Duplicate the 3rd item and push it on top	                [c, e, c − d]	                         [c − d, c, e, c − d]
// 7	GT	Pop a (top) and b (next), push 1 if a > b (unsigned), else 0	[c − d, c, e, c − d]	                 [(c − d) > c ? 1 : 0, e, c − d]
// 8	PUSH2	Read next 2 bytes as a big-endian immediate and push it	    [(c − d) > c ? 1 : 0, e, c − d]	         [imm16, (c − d) > c ? 1 : 0, e, c − d]

pub(super) fn swap2_swap1_dup3_sub_swap2_dup3_gt_push2<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 8 * gas::VERYLOW);

    // Grab mutable refs to [third, second, top] (deep -> shallow).
    backn!([third, second, top], context.interpreter);

    // Snapshot originals before we overwrite.
    let orig_top = *top; // t1
    let orig_second = *second; // t2
    let orig_third = *third; // t3

    // Final third = (t1 - t2) with EVM wrap semantics.
    *third = orig_top.wrapping_sub(orig_second);

    // Final second = original third.
    *second = orig_third;

    // Final top = ((t1 - t2) > t1) ? 1 : 0 (unsigned).
    *top = if *third > orig_top {
        U256::ONE
    } else {
        U256::ZERO
    };

    // Skip 7 remaining opcodes after the first one (auto-jump handles the first)
    context.interpreter.bytecode.relative_jump(7);

    // PUSH2: read immediate and push.
    let imm = context.interpreter.bytecode.read_slice(2);
    let value = U256::from_be_slice(imm);
    push!(context.interpreter, value);

    // Skip the 2 immediate bytes.
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: SWAP1 DUP2
/// Swaps top two elements then duplicates the second element
/// Fused: SWAP1 ; DUP2
/// Start: [a, b, c, …]
//
// #	Opcode	What it does	        Before	         After
// 1	SWAP1	Swap top & 2nd	        [a, b, c, …]	[b, a, c, …]
// 2	DUP2	Duplicate 2nd to top	[b, a, c, …]	[a, b, a, c, …]
pub(super) fn swap1_dup2<WIRE: InterpreterTypes, H: ?Sized>(
    // todo needs improvement
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, 2 * gas::VERYLOW);

    // backn! handles underflow + early return; faster than len() + two calls
    backn!([second, top], context.interpreter);

    // SWAP1
    core::mem::swap(top, second);

    // DUP2 (after swap, 2nd-from-top is *second)
    let copy = *second;
    push!(context.interpreter, copy);

    // skip the single padding byte after the fused opcode
    context.interpreter.bytecode.relative_jump(1);
}

/// Fused instruction: SHR SHR DUP1 MUL DUP1
/// Performs two right shifts, duplicates result, multiplies, then duplicates again
/// Fused: SHR ; SHR ; DUP1 ; MUL ; DUP1
/// | # | Opcode | What it does     | Before                | After                                   |
// | - | ------ | ---------------- | --------------------- | --------------------------------------- |
// | 1 | `SHR`  | `(v1 >> s1)`     | `[s1, v1, s2, v2, …]` | `[r1, s2, v2, …]` where `r1 = v1 >> s1` |
// | 2 | `SHR`  | `s2 >> r1`       | `[r1, s2, v2, …]`     | `[r2, v2, …]` where `r2 = s2 >> r1`     |
// | 3 | `DUP1` | duplicate top    | `[r2, v2, …]`         | `[r2, r2, v2, …]`                       |
// | 4 | `MUL`  | multiply top two | `[r2, r2, v2, …]`     | `[r2*r2, v2, …]`                        |
// | 5 | `DUP1` | duplicate top    | `[r2*r2, v2, …]`      | `[r2*r2, r2*r2, v2, …]`                 |
pub(super) fn shr_shr_dup1_mul_dup1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // 2*SHR(VL) + DUP1(VL) + MUL(LOW) + DUP1(VL)
    // gas!(context.interpreter, 4 * gas::VERYLOW + gas::LOW);

    // First SHR: pop shift, value -> r1
    popn!([s1, v1], context.interpreter);
    let r1 = if s1 < U256::from(256) {
        let n = s1.as_limbs()[0] as usize;
        v1 >> n
    } else {
        U256::ZERO
    };

    // Second SHR: apply r1 as shift to current top (s2); below it is v2
    backn!([_v2, s2], context.interpreter); // v2 = second, s2 = top
    *s2 = if r1 < U256::from(256) {
        let n = r1.as_limbs()[0] as usize;
        *s2 >> n
    } else {
        U256::ZERO
    };

    // DUP1; MUL → square top
    let t = *s2;
    *s2 = t.wrapping_mul(t);

    // Final DUP1
    if !context.interpreter.stack.dup(1) {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }

    // Five single-byte opcodes
    context.interpreter.bytecode.relative_jump(4);
}

///Start: [a, b, c, d, e, …]
///| # | Opcode  | What it does   | Before               | After                |
// | - | ------- | -------------- | -------------------- | -------------------- |
// | 1 | `SWAP3` | swap top & 4th | `[a, b, c, d, e, …]` | `[d, b, c, a, e, …]` |
// | 2 | `POP`   | drop top       | `[d, b, c, a, e, …]` | `[b, c, a, e, …]`    |
// | 3 | `POP`   | drop top       | `[b, c, a, e, …]`    | `[c, a, e, …]`       |
// | 4 | `POP`   | drop top       | `[c, a, e, …]`       | `[a, e, …]`          |
/// Fused: SWAP3 ; POP ; POP ; POP
pub(super) fn swap3_pop_pop_pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // gas!(context.interpreter, gas::VERYLOW + 3 * gas::BASE);

    // Pop original top four: a, b, c, d
    popn!([a, _b, _c, _d], context.interpreter);

    // Push back original top (a); stack now [a, e, …]
    push!(context.interpreter, a);

    // Four single-byte opcodes
    context.interpreter.bytecode.relative_jump(3);
}

// /// Fused instruction: SUB SLT ISZERO PUSH2
// /// Performs subtraction, signed less than, is zero check, then pushes 2-byte immediate
pub(super) fn sub_slt_iszero_push2<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // SUB (verylow) + SLT (verylow) + ISZERO (verylow) + PUSH2 (verylow) = 4 * VERYLOW
    // gas!(context.interpreter, 4 * gas::VERYLOW);

    // Grab top 3 stack items by REF:
    // z_slot = 3rd from top (will hold the final result),
    // y_slot = 2nd from top,
    // x_slot = top.
    backn!([z_slot, y_slot, x_slot], context.interpreter);

    // Copy values for arithmetic
    let x = *x_slot;
    let y = *y_slot;
    let z = *z_slot;

    // SUB: x - y
    let sub_result = x.wrapping_sub(y);

    // SLT (signed): (x - y) < z  ? 1 : 0
    let slt = if i256_cmp(&sub_result, &z) == core::cmp::Ordering::Less {
        U256::ONE
    } else {
        U256::ZERO
    };

    // ISZERO
    let iszero = if slt.is_zero() { U256::ONE } else { U256::ZERO };

    // Overwrite the 3rd-from-top slot with the final result,
    // then drop the top two original slots (x, y) to reduce 3→1.
    *z_slot = iszero;
    popn!([_drop1, _drop2], context.interpreter);

    // With step auto-jump: step advances 1, then we advance 3 to reach immediate bytes
    context.interpreter.bytecode.relative_jump(3);

    // Execute PUSH2: read the 2-byte immediate and push it
    let imm = context.interpreter.bytecode.read_slice(2);
    let value = U256::from_be_slice(imm);
    push!(context.interpreter, value);

    // Skip the immediate bytes to end of fused instruction
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused instruction: DUP11 MUL DUP3 SUB MUL DUP1
/// Duplicates 11th element, multiplies, duplicates 3rd, subtracts, multiplies, duplicates result
// Fused: DUP11 ; MUL ; DUP3 ; SUB ; MUL ; DUP1
/// | # | Opcode  | What it does     | Before                   | After                                       |
// | - | ------- | ---------------- | ------------------------ | ------------------------------------------- |
// | 1 | `DUP11` | copy 11th to top | `[a1,a2,a3,…,a11,a12,…]` | `[a11,a1,a2,a3,…,a11,a12,…]`                |
// | 2 | `MUL`   | multiply top two | `[a11,a1,a2,…]`          | `[m,a2,a3,…,a11,a12,…]` where `m=a11*a1`    |
// | 3 | `DUP3`  | copy 3rd to top  | `[m,a2,a3,…]`            | `[a3,m,a2,a3,a4,…]`                         |
// | 4 | `SUB`   | `a3 - m`         | `[a3,m,a2,a3,…]`         | `[a3-m,a2,a3,a4,…]`                         |
// | 5 | `MUL`   | `(a3-m)*a2`      | `[a3-m,a2,a3,…]`         | `[r,a3,a4,…,a11,a12,…]` where `r=(a3-m)*a2` |
// | 6 | `DUP1`  | duplicate top    | `[r,a3,a4,…]`            | `[r,r,a3,a4,…,a11,a12,…]`                   |
/// Fused: DUP11 ; MUL ; DUP3 ; SUB ; MUL ; DUP1
pub(super) fn dup11_mul_dup3_sub_mul_dup1<WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    // DUP11(VL) + MUL(LOW) + DUP3(VL) + SUB(VL) + MUL(LOW) + DUP1(VL)
    // gas!(context.interpreter, 4 * gas::VERYLOW + 2 * gas::LOW);

    let len = context.interpreter.stack.len();
    if len < 11 {
        context.interpreter.halt(InstructionResult::StackUnderflow);
        return;
    }

    // Grab a11 (11th from top) by value
    let a11 = context.interpreter.stack.data()[len - 11];

    // Pop the two that will be consumed by the two MULs (a1, a2)
    popn!([a1, a2], context.interpreter);

    // Current top is a3; compute r = (a3 - a11*a1) * a2 without disturbing a3 underneath result.
    backn!([a3], context.interpreter);
    let m = a11.wrapping_mul(a1);
    let r = a3.wrapping_sub(m).wrapping_mul(a2);

    // Push r on top (above a3), then DUP1 to match final shape [r, r, a3, …]
    push!(context.interpreter, r);
    if !context.interpreter.stack.dup(1) {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }

    // Five remaining opcodes after the first one (auto-jump handles the first)
    context.interpreter.bytecode.relative_jump(5);
}

#[cfg(test)]
mod fused_tests {
    use super::*;
    use crate::instructions::{arithmetic, bitwise, control, memory, stack, system};
    use crate::interpreter::{EthInterpreter, ExtBytecode, Interpreter};
    use crate::InstructionContext;
    use bitvec::{bitvec, order::Lsb0, vec::BitVec};
    use bytecode::{Bytecode, JumpTable};
    use primitives::Bytes;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

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
        // Set up gas for testing (enough for all operations)
        i.gas = crate::Gas::new(1000000);
        let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; len];
        unsafe { jumps.set_unchecked(jump_loc, true) }
        let mut v = vec![0u8; len];
        for i in 0..len - 1 {
            v[i] = i as u8;
        }

        let dummy = Bytecode::new_analyzed(Bytes::from(v), len, JumpTable::new(jumps));
        i.bytecode = ExtBytecode::new(dummy);
        i
    }

    fn run<F>(mut interp: Interp, f: F) -> (Interp, usize)
    where
        F: FnOnce(&mut Interp),
    {
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
            and_swap1_pop_swap2_swap1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, _) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            bitwise::bitand(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            swap1_pop_swap2_swap1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, _) = run(make_interp(10), |ip| {
            for n in 0..3 {
                let _ = ip.stack.push(U256::from(n));
            }
            let _ = ip.stack.push(U256::from(7));
            let _ = ip.stack.push(U256::from(5));
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            swap2_swap1_pop_jump(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // println!("{:?}, {:?}", ip.stack, ip.bytecode.pc());
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 7), |ip| {
            // stack: [dest, keep, discard, extra...]
            let _ = ip.stack.push(U256::from(7)); // jump dest
            let _ = ip.stack.push(U256::from(0xaa));
            let _ = ip.stack.push(U256::from(0xbb));
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            control::jump(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            pop_swap2_swap1_pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, _pc2) = run(make_interp(10), |ip| {
            for n in 0..4 {
                let _ = ip.stack.push(U256::from(n));
            }
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            swap2_pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        let (interp2, pc2) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
        });

        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push2_jump() {
        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            push2_jump(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            control::jump(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push2_jumpi() {
        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(5));
            push2_jumpi(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(5));
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            control::jumpi(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc, 1);
        assert_eq!(pc2, 1);
        assert_eq!(interp.stack, interp2.stack);

        let (interp, pc) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(0));
            push2_jumpi(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (interp2, pc2) = run(make_interp_with_jump(10, 1), |ip| {
            let _ = ip.stack.push(U256::from(0));
            // println!("{:?}", ip.bytecode.pc());
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            control::jumpi(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc, 3);
        assert_eq!(pc2, 3);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_push1_add() {
        let (mut interp, pc) = run(make_interp(4), |ip| {
            let _ = ip.stack.push(U256::ONE);
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![5, 0, 0, 0].into()));
            push1_add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(6));
        assert_eq!(pc, 2);
    }

    #[test]
    fn test_pop2() {
        let (interp, pc) = run(make_interp(2), |ip| {
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            pop2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert!(interp.stack.is_empty());
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_dup2_lt() {
        let (mut interp, pc) = run(make_interp(2), |ip| {
            let _ = ip.stack.push(U256::from(3));
            let _ = ip.stack.push(U256::from(4));
            dup2_lt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::ONE); // 4 < 3 false, but after swap → comparison 3<4 ⇒ true
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_push1_shl() {
        let (mut interp, pc) = run(make_interp_with_jump(3, 0), |ip| {
            ip.bytecode.relative_jump(1);
            let _ = ip.stack.push(U256::from(4));
            push1_shl(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(8));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_push1_dup1() {
        let (mut interp, pc) = run(make_interp_with_jump(3, 0), |ip| {
            ip.bytecode.relative_jump(1);
            push1_dup1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            swap1_pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            pop_jump(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(interp.stack.top().unwrap(), &U256::from(2));
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_jump_if_zero() {
        let (interp_true, pc_true) = run(make_interp_with_jump(260, 258), |ip| {
            let _ = ip.stack.push(U256::ZERO);
            jump_if_zero(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc_true, 258);

        let (_interp_false, pc_false) = run(make_interp_with_jump(10, 0), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_legacy(Bytes::from(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ])));
            let _ = ip.stack.push(U256::ONE);
            jump_if_zero(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc_false, 4);
    }

    #[test]
    fn test_push1_push1() {
        let (mut interp, pc) = run(make_interp(5), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(vec![0x05, 0x00, 0x07, 0, 0].into()));
            push1_push1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            iszero_push2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::from(0x1234u16));
        let second = interp.stack.data().get(interp.stack.len() - 2).unwrap();
        assert_eq!(*second, U256::ONE);
        assert_eq!(pc, 3);
    }

    #[test]
    fn test_push1_push1_push1_shl_sub() {
        let (mut interp, pc) = run(make_interp(10), |ip| {
            // immediates: imm1=2, imm2=3, imm3=1 => (3<<1)-2 = 4
            ip.bytecode =
                ExtBytecode::new(Bytecode::new_raw(vec![2, 0, 3, 0, 1, 0, 0, 0, 0, 0].into()));
            push1_push1_push1_shl_sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            // immediates: imm1=2, imm2=3, imm3=1 => (3<<1)-2 = 4
            ip.bytecode =
                ExtBytecode::new(Bytecode::new_raw(vec![2, 0, 3, 0, 1, 0, 0, 0, 0, 0].into()));
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::shl(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
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
            and_dup2_add_swap1_dup2_lt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            let _ = ip.stack.push(U256::from(3)); // c (bottom of 3 values)
            let _ = ip.stack.push(U256::from(2)); // b
            let _ = ip.stack.push(U256::from(1)); // a (top)
            arithmetic::add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::lt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::ZERO);
        let second = interp.stack.data().get(interp.stack.len() - 2).unwrap();
        assert_eq!(*second, U256::from(6u8));
        assert_eq!(pc, 5);
        assert_eq!(pc2, 5);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_dup2_mstore_push1_add() {
        // initial stack: offset(0x20), val(0x0a)
        let (mut interp, pc) = run(make_interp(10), |ip| {
            ip.bytecode =
                ExtBytecode::new(Bytecode::new_raw(vec![0, 0, 7, 0, 0, 0, 0, 0, 0, 0].into()));
            let _ = ip.stack.push(U256::from(0x20u64));
            let _ = ip.stack.push(U256::from(0x0au8));
            dup2_mstore_push1_add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            ip.bytecode =
                ExtBytecode::new(Bytecode::new_raw(vec![0, 0, 7, 0, 0, 0, 0, 0, 0, 0].into()));
            let _ = ip.stack.push(U256::from(0x20u64));
            let _ = ip.stack.push(U256::from(0x0au8));
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            memory::mstore(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        // offset should be 0x20+7=0x27 at top
        assert_eq!(interp.stack.top().unwrap(), &U256::from(0x27u64));
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_dup1_push4_eq_push2() {
        // const value 0x01020304, dest=0x0003
        let bytes = vec![0, 0x01, 0x02, 0x03, 0x04, 0, 0x00, 0x03, 0];
        let (mut interp, pc) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            let _ = ip.stack.push(U256::from_be_slice(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0x01, 0x02, 0x03, 0x04,
            ]));
            dup1_push4_eq_push2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (mut interp2, pc2) = run(make_interp(10), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            let _ = ip.stack.push(U256::from_be_slice(&[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0x01, 0x02, 0x03, 0x04,
            ]));
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<4, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::eq(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        // top dest
        assert_eq!(interp.stack.top().unwrap(), &U256::from(768u16));
        // eq result should be ONE
        let second = interp.stack.data().get(interp.stack.len() - 2).unwrap();
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
            swap1_push1_dup1_not_swap2_add_and_dup2_add_swap1_dup2_lt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let (mut interp2, pc2) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // initial stack top a=1, b=2 (note reversed order top-first)
            let _ = ip.stack.push(U256::from(2)); // b (top)
            let _ = ip.stack.push(U256::from(1)); // a (second)
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::not(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::bitand(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            arithmetic::add(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::lt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        assert_eq!(pc, pc2);
        assert_eq!(interp.stack, interp2.stack);
    }

    use primitives::hex::decode;
    #[test]
    fn test_push1_calldataload_push1_shr_dup1_push4_gt_push2() {
        // Setup immediate values: offset=0, shift=0, const=0x00000002 (> x), dest=0x0004
        let mut bytes = vec![0u8; 16];
        let input =
            decode("70a082310000000000000000000000001000000000000000000000000000000000000001")
                .unwrap();
        bytes[0] = 0; // offset
        bytes[3] = 0xe0; // shift
        bytes[7] = 0x89;
        bytes[8] = 0x3d;
        bytes[9] = 0x20;
        bytes[10] = 0xe8; // const
        bytes[13] = 0;
        bytes[14] = 0xad; // dest =4

        let (mut interp, pc) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // Provide calldata: 32 bytes zero so x=0
            ip.input.input = CallInput::Bytes(Bytes::from(input.clone()));
            push1_calldataload_push1_shr_dup1_push4_gt_push2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        let (mut interp2, pc2) = run(make_interp(20), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(bytes.clone().into()));
            // Provide calldata: 32 bytes zero so x=0
            ip.input.input = CallInput::Bytes(Bytes::from(input));
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            system::calldataload(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<4, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            bitwise::gt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Stack top dest (0x0004)
        let top = interp.stack.top().unwrap();
        assert_eq!(*top, U256::from(173u16));
        // second should be ONE (const > x)
        let second = interp.stack.data().get(interp.stack.len() - 2).unwrap();
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
            swap2_swap1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        let (mut interp2, pc2) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0));
            let _ = ip.stack.push(U256::from(1));
            let _ = ip.stack.push(U256::from(2));
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(1);
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        assert_eq!(pc, pc2);
        assert_eq!(pc, 1);
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_snop() {
        let (mut interp, pc) = run(make_interp(1), |ip| {
            let _ = ip.stack.push(U256::from(123));
            snop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        assert_eq!(pc, 0);
        assert_eq!(interp.stack.top().unwrap(), &U256::from(123u64));
    }

    // ============================ Tests for the 7 new fused functions ============================

    #[test]
    fn test_dup3_and() {
        // passing
        // Test fused function
        let (mut interp_fused, pc_fused) = run(make_interp(2), |ip| {
            // Setup stack: [0x0F, 0xF0, 0x33] (bottom to top)
            let _ = ip.stack.push(U256::from(0x0Fu8)); // 3rd from top
            let _ = ip.stack.push(U256::from(0xF0u8)); // 2nd from top
            let _ = ip.stack.push(U256::from(0x33u8)); // top
            dup3_and(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Test reference implementation: DUP3 + AND
        let (mut interp_ref, _pc_ref) = run(make_interp(2), |ip| {
            // Setup identical stack
            let _ = ip.stack.push(U256::from(0x0Fu8));
            let _ = ip.stack.push(U256::from(0xF0u8));
            let _ = ip.stack.push(U256::from(0x33u8));

            // DUP3: duplicate 3rd element to top
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // AND: bitwise and of top two elements
            bitwise::bitand(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Gas comparison
        let gas_fused = interp_fused.gas.spent();
        let gas_ref = interp_ref.gas.spent();

        // Functional correctness
        assert_eq!(pc_fused, 1);
        assert_eq!(interp_fused.stack.top().unwrap(), &U256::from(0x03u8));
        assert_eq!(
            interp_fused.stack, interp_ref.stack,
            "Stack states should be identical"
        );

        // Gas efficiency: fused should use same or less gas
        assert!(
            gas_fused <= gas_ref,
            "Fused function used {} gas, reference used {} gas",
            gas_fused,
            gas_ref
        );
    }

    #[test]
    fn test_swap1_dup2() {
        // passing
        // Test fused function
        let (mut interp_fused, pc_fused) = run(make_interp(2), |ip| {
            // Setup stack: [1, 2] (bottom to top)
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));
            swap1_dup2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Test reference implementation: SWAP1 + DUP2
        let (mut interp_ref, _pc_ref) = run(make_interp(2), |ip| {
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Gas comparison
        let gas_fused = interp_fused.gas.spent();
        let gas_ref = interp_ref.gas.spent();

        // Functional correctness
        assert_eq!(pc_fused, 1);
        assert_eq!(interp_fused.stack.len(), 3);
        assert_eq!(interp_fused.stack.top().unwrap(), &U256::from(2u8)); // duplicated 2nd element
        assert_eq!(
            interp_fused.stack, interp_ref.stack,
            "Stack states should be identical"
        );

        // Gas efficiency: fused should use same or less gas
        assert!(
            gas_fused <= gas_ref,
            "Fused function used {} gas, reference used {} gas",
            gas_fused,
            gas_ref
        );
    }

    #[test]
    fn test_shr_shr_dup1_mul_dup1_0() {
        // passing
        let (mut interp, pc) = run(make_interp(5), |ip| {
            // Setup stack to match Go test: [1,2,3,3,1,2,3,3,1,2,3,3] (12 elements)
            for i in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }
            shr_shr_dup1_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Run the equivalent individual operations to compare
        let (mut interp2, pc2) = run(make_interp(5), |ip| {
            for i in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }
            // SHR SHR DUP1 MUL DUP1
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        assert_eq!(interp.stack.len(), interp2.stack.len());
        assert_eq!(interp.stack, interp2.stack);
    }

    #[test]
    fn test_shr_shr_dup1_mul_dup1() {
        // Initial stack (bottom → top):
        //   0x10 (value₂) , 0x02 (shift₂) , 0x08 (value₁) , 0x01 (shift₁)
        let (mut interp_fused, pc_fused) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0x10u8));
            let _ = ip.stack.push(U256::from(0x02u8));
            let _ = ip.stack.push(U256::from(0x08u8));
            let _ = ip.stack.push(U256::from(0x01u8));
            shr_shr_dup1_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        let (mut interp_ref, pc_ref) = run(make_interp(5), |ip| {
            let _ = ip.stack.push(U256::from(0x10u8));
            let _ = ip.stack.push(U256::from(0x02u8));
            let _ = ip.stack.push(U256::from(0x08u8));
            let _ = ip.stack.push(U256::from(0x01u8));
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // After the sequence the stack should be:
        //   bottom … 0x10 , 0x00 , 0x00 (top two equal) and pc advanced by 4
        assert_eq!(interp_fused.stack, interp_ref.stack);
        assert_eq!(pc_fused, 4);
    }

    #[test]
    fn test_swap3_pop_pop_pop() {
        // passing
        // Test fused function
        let (mut interp_fused, pc_fused) = run(make_interp(4), |ip| {
            // Setup stack: [1, 2, 3, 4] (bottom to top)
            let _ = ip.stack.push(U256::from(1u8)); // 4th from top (will become top after SWAP3)
            let _ = ip.stack.push(U256::from(2u8)); // 3rd from top (will be popped)
            let _ = ip.stack.push(U256::from(3u8)); // 2nd from top (will be popped)
            let _ = ip.stack.push(U256::from(4u8)); // top (will be popped)
            swap3_pop_pop_pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Test reference implementation: individual operations
        let (mut interp_ref, _pc_ref) = run(make_interp(4), |ip| {
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));
            let _ = ip.stack.push(U256::from(3u8));
            let _ = ip.stack.push(U256::from(4u8));
            // SWAP3: exchange top with 4th element
            stack::swap::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // POP: remove 3 elements
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Gas comparison
        let gas_fused = interp_fused.gas.spent();
        let gas_ref = interp_ref.gas.spent();

        // Functional correctness
        assert_eq!(pc_fused, 3);
        assert_eq!(interp_fused.stack.len(), interp_ref.stack.len());
        assert_eq!(
            interp_fused.stack, interp_ref.stack,
            "Stack states should be identical"
        );

        // Gas efficiency: fused should use same or less gas
        assert!(
            gas_fused <= gas_ref,
            "Fused function used {} gas, reference used {} gas",
            gas_fused,
            gas_ref
        );
    }

    #[test]
    fn test_sub_slt_iszero_push2_matches_reference() {
        use primitives::U256;
        use std::time::Instant;
        // one of these two, depending on your re-exports:
        use crate::interpreter::ExtBytecode;
        use bytecode::Bytecode;

        // Bytecode layout so that after skipping 3 bytes, the next 2 are the PUSH2 immediate
        // (dummy leading bytes stand in for the SUB/SLT/ISZERO opcodes in real bytecode)
        let make_bc =
            || ExtBytecode::new(Bytecode::new_raw(vec![0x00, 0x00, 0x00, 0x12, 0x34].into()));

        // Build identical initial stacks: bottom→top = [ z=2, y=5, x=3 ]
        let build_stack = |ip: &mut Interpreter| {
            let _ = ip.stack.push(U256::from(2u8)); // z
            let _ = ip.stack.push(U256::from(5u8)); // y
            let _ = ip.stack.push(U256::from(3u8)); // x
        };

        // Fused - with timing
        let start_fused = Instant::now();
        let (interp_fused, pc_fused) = run(make_interp(7), |ip| {
            ip.bytecode = make_bc();
            build_stack(ip);
            // Simulate step's auto-jump: advance PC by 1 to match step execution
            // ip.bytecode.relative_jump(1);
            sub_slt_iszero_push2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });
        let fused_duration = start_fused.elapsed();

        // Reference path: SUB; SLT; ISZERO; PUSH2 - with timing
        let start_ref = Instant::now();
        let (interp_ref, _pc_ref) = run(make_interp(7), |ip| {
            ip.bytecode = make_bc();
            build_stack(ip);

            // SUB (x - y)
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // SLT (signed): z < (x-y)
            crate::instructions::bitwise::slt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // ISZERO
            bitwise::iszero(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });

            // Skip to the PUSH2 immediate (at positions 3-4 in 5-byte bytecode)
            ip.bytecode.relative_jump(3);
            // PUSH2 immediate (read 2 bytes and push)
            let imm = ip.bytecode.read_slice(2);
            let val = U256::from_be_slice(imm);
            let _ = ip.stack.push(val);
        });
        let ref_duration = start_ref.elapsed();

        // Print timing results
        println!("Fused approach time: {:?}", fused_duration);
        println!("Non-fused (reference) approach time: {:?}", ref_duration);
        println!(
            "Performance ratio (ref/fused): {:.2}",
            ref_duration.as_nanos() as f64 / fused_duration.as_nanos() as f64
        );

        // With step simulation: PC=1 + relative_jump(2) + relative_jump(2) = PC=5
        assert_eq!(pc_fused, 5);

        // Full-stack equality
        assert_eq!(interp_fused.stack, interp_ref.stack);

        // Optional explicit checks for clarity:
        let data = interp_fused.stack.data();
        assert_eq!(data.len(), 2); // net -1 effect
        assert_eq!(data[0], U256::ZERO); // ISZERO(SLT(2, 3-5)) = ISZERO(1) = 0
        assert_eq!(data[1], U256::from(0x1234u16)); // PUSH2
    }

    #[test]
    fn test_dup11_mul_dup3_sub_mul_dup1_matches_reference() {
        use primitives::U256;

        // Build identical initial stacks: [1,2,3,4,5,6,7,8,9,10,11,12] bottom to top
        let build_stack = |ip: &mut Interpreter| {
            for i in 0..12 {
                let _ = ip.stack.push(U256::from(i + 1)); // [1,2,3,...,12] bottom to top
            }
        };

        // Fused
        let (interp_fused, pc_fused) = run(make_interp(6), |ip| {
            build_stack(ip);
            dup11_mul_dup3_sub_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Reference path: DUP11; MUL; DUP3; SUB; MUL; DUP1
        let (interp_ref, _pc_ref) = run(make_interp(6), |ip| {
            build_stack(ip);

            // DUP11: duplicate 11th element from top
            stack::dup::<11, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // MUL: multiply top two elements
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // DUP3: duplicate 3rd element from top
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // SUB: subtract top two elements
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // MUL: multiply top two elements
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            // DUP1: duplicate top element
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Fused must skip 5 bytes (as implemented in the function)
        assert_eq!(pc_fused, 5);

        // Full-stack equality
        assert_eq!(interp_fused.stack, interp_ref.stack);

        // Optional explicit checks for stack structure
        let data = interp_fused.stack.data();
        // Net stack effect: initial 12 elements, DUP11 (+1), MUL (-1), DUP3 (+1), SUB (-1), MUL (-1), DUP1 (+1) = 12

        // Verify DUP1 worked correctly - last two elements should be identical
        assert_eq!(data[data.len() - 1], data[data.len() - 2]);
    }

    #[test]
    fn test_dup11_mul_dup3_sub_mul_dup1_matches_reference2() {
        use primitives::U256;

        // Build identical initial stacks: [1..12] bottom→top
        let build_stack = |ip: &mut Interpreter| {
            for i in 1..=12 {
                let _ = ip.stack.push(U256::from(i));
            }
        };

        // Fused
        let (interp_fused, pc_fused) = run(make_interp(6), |ip| {
            build_stack(ip);
            dup11_mul_dup3_sub_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Reference: DUP11; MUL; DUP3; SUB; MUL; DUP1
        let (interp_ref, _pc_ref) = run(make_interp(6), |ip| {
            build_stack(ip);
            stack::dup::<11, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Fused must skip 5 bytes (we executed the whole 6-op bundle)
        assert_eq!(pc_fused, 5);

        // Full-stack equality
        assert_eq!(interp_fused.stack, interp_ref.stack);

        // Sanity: last two equal (DUP1)
        let data = interp_fused.stack.data();
        assert_eq!(data[data.len() - 1], data[data.len() - 2]);
    }

    #[test]
    fn test_swap2_swap1_dup3_sub_swap2_dup3_gt_push2() {
        // Test fused function
        let (interp_fused, pc_fused) = run(make_interp(11), |ip| {
            // Setup bytecode to match Go test: [0x91,0x90,0x82,0x3,0x91,0x82,0x11,0x61,0x1,0x2]
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(
                vec![0x91, 0x90, 0x82, 0x3, 0x91, 0x82, 0x11, 0x61, 0x1, 0x2].into(),
            ));

            // Setup stack to match Go test: [1,2,3,3,1,2,3,3,1,2,3,3] (12 elements)
            for _i in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }

            // Simulate step's auto-jump: advance PC by 1 to match step execution
            ip.bytecode.relative_jump(1);
            swap2_swap1_dup3_sub_swap2_dup3_gt_push2(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Test reference implementation: individual operations
        let (interp_ref, _pc_ref) = run(make_interp(11), |ip| {
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(
                vec![0x91, 0x90, 0x82, 0x3, 0x91, 0x82, 0x11, 0x61, 0x1, 0x2].into(),
            ));
            for _i in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }
            // SWAP2 SWAP1 DUP3 SUB SWAP2 DUP3 GT PUSH2
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            bitwise::gt(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
            ip.bytecode.relative_jump(8); // skip to PUSH2 immediate
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: ip,
            });
        });

        // Gas comparison
        let gas_fused = interp_fused.gas.spent();
        let gas_ref = interp_ref.gas.spent();

        // Functional correctness
        assert_eq!(pc_fused, 10); // Start at 1 + relative_jump(7) + relative_jump(2) = 1+7+2 = 10
        assert_eq!(interp_fused.stack.len(), interp_ref.stack.len());
        assert_eq!(
            interp_fused.stack, interp_ref.stack,
            "Stack states should be identical"
        );

        // Gas efficiency: fused should use same or less gas
        assert!(
            gas_fused <= gas_ref,
            "Fused function used {} gas, reference used {} gas",
            gas_fused,
            gas_ref
        );
    }

    ///// Timed //////

    #[test]
    fn time_swap2_swap1_dup3_sub_swap2_dup3_gt_push2_timing() {
        // Enough iterations to smooth noise; adjust if slow on your box.
        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            // Setup fresh interpreter + state (excluded from timing)
            let mut ip = make_interp(11);
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(
                vec![0x91, 0x90, 0x82, 0x03, 0x91, 0x82, 0x11, 0x61, 0x01, 0x02].into(),
            ));
            for _ in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }

            // Time only the fused op
            let start = Instant::now();
            swap2_swap1_dup3_sub_swap2_dup3_gt_push2(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            // Prevent UB/over-optimizing away
            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(11);
            ip.bytecode = ExtBytecode::new(Bytecode::new_raw(
                vec![0x91, 0x90, 0x82, 0x03, 0x91, 0x82, 0x11, 0x61, 0x01, 0x02].into(),
            ));
            for _ in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }

            let start = Instant::now();
            // SWAP2 SWAP1 DUP3 SUB SWAP2 DUP3 GT PUSH2
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::swap::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            bitwise::gt(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });

            // Skip 7 fused bytes + 1 PUSH2 opcode before pushing 2-byte immediate.
            ip.bytecode.relative_jump(8);
            stack::push::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        let fused_avg_ns = fused_total.as_nanos() as f64 / ITERS as f64;
        let ref_avg_ns = ref_total.as_nanos() as f64 / ITERS as f64;

        eprintln!(
            "FUSED   total = {:?}, avg = {:.2} ns/iter",
            fused_total, fused_avg_ns
        );
        eprintln!(
            "REF     total = {:?}, avg = {:.2} ns/iter",
            ref_total, ref_avg_ns
        );

        // (Optional) sanity check: fused should not be slower than reference
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:.2} ns vs {:.2} ns",
            fused_avg_ns,
            ref_avg_ns
        );
    }

    #[test]
    fn time_dup3_and_timing() {
        // dup3_and FUSED = 14.231854ms, REF = 35.497975ms
        use primitives::U256;

        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(2);
            let _ = ip.stack.push(U256::from(0x0Fu8));
            let _ = ip.stack.push(U256::from(0xF0u8));
            let _ = ip.stack.push(U256::from(0x33u8));

            let start = Instant::now();
            dup3_and(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(2);
            let _ = ip.stack.push(U256::from(0x0Fu8));
            let _ = ip.stack.push(U256::from(0xF0u8));
            let _ = ip.stack.push(U256::from(0x33u8));

            let start = Instant::now();
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            bitwise::bitand(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!("dup3_and FUSED = {:?}, REF = {:?}", fused_total, ref_total);
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:?} vs {:?}",
            fused_total,
            ref_total
        );
    }

    #[test]
    fn time_swap1_dup2_timing() {
        // swap1_dup2 FUSED = 11.490199ms, REF = 9.365927ms
        use primitives::U256;

        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(2);
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));

            let start = Instant::now();
            swap1_dup2(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(2);
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));

            let start = Instant::now();
            stack::swap::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<2, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!(
            "swap1_dup2 FUSED = {:?}, REF = {:?}",
            fused_total, ref_total
        );
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:?} vs {:?}",
            fused_total,
            ref_total
        );
    }

    #[test]
    fn time_shr_shr_dup1_mul_dup1_bulk_timing() {
        // shr_shr_dup1_mul_dup1 (bulk) FUSED = 52.078976ms, REF = 62.054462ms
        use primitives::U256;

        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(5);
            for _ in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }

            let start = Instant::now();
            shr_shr_dup1_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(5);
            for _ in 0..4 {
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(3));
            }

            let start = Instant::now();
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            bitwise::shr(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!(
            "shr_shr_dup1_mul_dup1 (bulk) FUSED = {:?}, REF = {:?}",
            fused_total, ref_total
        );
        // Allow up to 5% performance regression to account for timing noise
        let tolerance = ref_total * 105 / 100;
        assert!(
            fused_total <= tolerance,
            "Fused significantly slower: {:?} vs {:?} (tolerance: {:?})",
            fused_total,
            ref_total,
            tolerance
        );
    }

    #[test]
    fn time_swap3_pop_pop_pop_timing() {
        // swap3_pop_pop_pop FUSED = 13.701338ms, REF = 18.307659ms
        use primitives::U256;

        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(4);
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));
            let _ = ip.stack.push(U256::from(3u8));
            let _ = ip.stack.push(U256::from(4u8));

            let start = Instant::now();
            swap3_pop_pop_pop(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(4);
            let _ = ip.stack.push(U256::from(1u8));
            let _ = ip.stack.push(U256::from(2u8));
            let _ = ip.stack.push(U256::from(3u8));
            let _ = ip.stack.push(U256::from(4u8));

            let start = Instant::now();
            stack::swap::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::pop(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!(
            "swap3_pop_pop_pop FUSED = {:?}, REF = {:?}",
            fused_total, ref_total
        );
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:?} vs {:?}",
            fused_total,
            ref_total
        );
    }

    #[test]
    fn time_sub_slt_iszero_push2_timing() {
        // sub_slt_iszero_push2 FUSED = 26.702218ms, REF = 32.368462ms
        use crate::interpreter::ExtBytecode;
        use bytecode::Bytecode;
        use primitives::U256;

        const ITERS: usize = 50_000;

        let make_bc =
            || ExtBytecode::new(Bytecode::new_raw(vec![0x00, 0x00, 0x00, 0x12, 0x34].into()));

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(7);
            ip.bytecode = make_bc();
            let _ = ip.stack.push(U256::from(2u8)); // z
            let _ = ip.stack.push(U256::from(5u8)); // y
            let _ = ip.stack.push(U256::from(3u8)); // x

            let start = Instant::now();
            sub_slt_iszero_push2(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(7);
            ip.bytecode = make_bc();
            let _ = ip.stack.push(U256::from(2u8)); // z
            let _ = ip.stack.push(U256::from(5u8)); // y
            let _ = ip.stack.push(U256::from(3u8)); // x

            let start = Instant::now();
            // SUB (x - y)
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            // SLT (signed): z < (x-y)
            crate::instructions::bitwise::slt(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            // ISZERO
            bitwise::iszero(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });

            // PUSH2 immediate at bytes 3..5
            ip.bytecode.relative_jump(3);
            let imm = ip.bytecode.read_slice(2);
            let val = U256::from_be_slice(imm);
            let _ = ip.stack.push(val);
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!(
            "sub_slt_iszero_push2 FUSED = {:?}, REF = {:?}",
            fused_total, ref_total
        );
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:?} vs {:?}",
            fused_total,
            ref_total
        );
    }

    #[test]
    fn time_dup11_mul_dup3_sub_mul_dup1_timing() {
        // dup11_mul_dup3_sub_mul_dup1 FUSED = 32.827196ms, REF = 45.886887ms
        use primitives::U256;

        const ITERS: usize = 50_000;

        // --- FUSED ---
        let mut fused_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(6);
            for i in 1..=12 {
                let _ = ip.stack.push(U256::from(i));
            }

            let start = Instant::now();
            dup11_mul_dup3_sub_mul_dup1(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            fused_total += start.elapsed();

            black_box(&ip);
        }

        // --- REFERENCE ---
        let mut ref_total = Duration::ZERO;
        for _ in 0..ITERS {
            let mut ip = make_interp(6);
            for i in 1..=12 {
                let _ = ip.stack.push(U256::from(i));
            }

            let start = Instant::now();
            stack::dup::<11, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<3, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            arithmetic::sub(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            arithmetic::mul(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            stack::dup::<1, _, _>(InstructionContext {
                host: &mut (),
                interpreter: &mut ip,
            });
            ref_total += start.elapsed();

            black_box(&ip);
        }

        eprintln!(
            "dup11_mul_dup3_sub_mul_dup1 FUSED = {:?}, REF = {:?}",
            fused_total, ref_total
        );
        assert!(
            fused_total <= ref_total,
            "Fused slower: {:?} vs {:?}",
            fused_total,
            ref_total
        );
    }

    // Helper function for PC tests
    fn make_interp_with_bytecode(bytes: Vec<u8>) -> Interp {
        let mut i = Interp::default_ext();
        let dummy = Bytecode::new_legacy(Bytes::from(bytes));
        i.bytecode = ExtBytecode::new(dummy);
        i
    }

    #[test]
    fn test_fused_instruction_pc_advancement() {
        // Test each fused function's PC advancement without immediates
        // Note: These are called DIRECTLY, not through step(), so there's no auto-jump
        // The functions use relative_jump(N-1) expecting to be called via step() which adds 1
        let test_cases: &[(&str, usize, fn(&mut Interp))] = &[
            ("DUP3AND", 1, |ip: &mut Interp| {
                // 2 opcodes, relative_jump(1) = PC 1
                let _ = ip.stack.push(U256::from(0x0F));
                let _ = ip.stack.push(U256::from(0xF0));
                let _ = ip.stack.push(U256::from(0x33));
                dup3_and(InstructionContext {
                    host: &mut (),
                    interpreter: ip,
                });
            }),
            ("SWAP1DUP2", 1, |ip: &mut Interp| {
                // 2 opcodes, relative_jump(1) = PC 1
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(2));
                swap1_dup2(InstructionContext {
                    host: &mut (),
                    interpreter: ip,
                });
            }),
            ("SHRSHRDUP1MULDUP1", 4, |ip: &mut Interp| {
                // 5 opcodes, relative_jump(4) = PC 4
                let _ = ip.stack.push(U256::from(3));
                let _ = ip.stack.push(U256::from(2));
                let _ = ip.stack.push(U256::from(1));
                let _ = ip.stack.push(U256::from(4));
                shr_shr_dup1_mul_dup1(InstructionContext {
                    host: &mut (),
                    interpreter: ip,
                });
            }),
            ("SWAP3POPPOPPOP", 3, |ip: &mut Interp| {
                // 4 opcodes, relative_jump(3) = PC 3
                for i in 1..=5 {
                    let _ = ip.stack.push(U256::from(i));
                }
                swap3_pop_pop_pop(InstructionContext {
                    host: &mut (),
                    interpreter: ip,
                });
            }),
            ("DUP11MULDUP3SUBMULDUP1", 5, |ip: &mut Interp| {
                // 6 opcodes, relative_jump(5) = PC 5
                for i in 1..=12 {
                    let _ = ip.stack.push(U256::from(i));
                }
                dup11_mul_dup3_sub_mul_dup1(InstructionContext {
                    host: &mut (),
                    interpreter: ip,
                });
            }),
        ];

        for &(name, expected_pc, test_fn) in test_cases {
            let (_interp, final_pc) = run(make_interp(20), test_fn);

            // PC should advance by N-1 for N-byte instructions (direct call, no auto-jump)
            assert_eq!(
                final_pc, expected_pc,
                "{}: PC should advance by {} bytes (direct call), but advanced by {} bytes",
                name, expected_pc, final_pc
            );
        }
    }
}
