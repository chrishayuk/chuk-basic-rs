//! The runtime that compiled BASIC programs call into.
//!
//! `chuk-basic --compile` turns a program into WebAssembly instructions for
//! its arithmetic and control flow; everything else - strings, PRINT and
//! INPUT, arrays, the built-in functions - is done by calling the functions
//! exported here. They are thin wrappers over `chuk_basic::rt`, the same
//! code the interpreter uses, so a compiled program prints exactly what the
//! interpreter would.
//!
//! This crate is built for `wasm32-wasip1` (see `scripts/build-runtime.sh`)
//! and the compiler copies the resulting module into every program it
//! produces, adding the program's own functions to it.
//!
//! Conventions: numbers are `f64`; a string is an `i32` handle (0 is the
//! empty string); nothing here returns an error - a runtime error prints
//! its message and ends the program.

mod state;

use chuk_basic::ast::{Builtin, DataItem};
use chuk_basic::rt::mat::{self, MatOp};
use chuk_basic::rt::using::format_using;
use chuk_basic::rt::{builtins, subscript, Value};

use state::{with, ForFrame, GosubFrame, Runtime, MAX_FN_DEPTH, MAX_GOSUB_DEPTH};

// ---- start-up -----------------------------------------------------------------

/// Called first: sets up the runtime with room for the program's variables.
#[no_mangle]
pub extern "C" fn rt_init(numbers: i32, strings: i32) {
    state::init(numbers as usize, strings as usize);
}

/// Text (string literals, DATA items, array names) arrives eight bytes at a
/// time, to be collected by the `rt_add_*` call that follows.
#[no_mangle]
pub extern "C" fn rt_bytes(chunk: i64, count: i32) {
    with(|rt| {
        rt.staged
            .extend_from_slice(&chunk.to_le_bytes()[..count as usize])
    });
}

/// A string literal; returns its handle, which lasts for the whole run.
#[no_mangle]
pub extern "C" fn rt_add_literal() -> i32 {
    with(|rt| {
        let text = rt.take_staged();
        rt.strings.keep(text)
    })
}

#[no_mangle]
pub extern "C" fn rt_add_data(quoted: i32) {
    with(|rt| {
        let text = rt.take_staged();
        rt.data.push(DataItem {
            text,
            quoted: quoted != 0,
        });
    });
}

/// Line `number` starts here, as far as RESTORE is concerned.
#[no_mangle]
pub extern "C" fn rt_data_line(number: i32) {
    with(|rt| rt.data.mark_line(number as u32));
}

/// Name the next array; arrays are numbered in the order they are named.
#[no_mangle]
pub extern "C" fn rt_add_array() {
    with(|rt| {
        let name = rt.take_staged();
        rt.array_names.push(name);
    });
}

/// Where the numeric variables live: variable `n` is the `f64` at this
/// address plus `8 * n`.
#[no_mangle]
pub extern "C" fn rt_num_base() -> i32 {
    with(|rt| rt.numbers.as_mut_ptr() as i32)
}

/// The program ran off its end.
#[no_mangle]
pub extern "C" fn rt_finish() {
    with(|rt| {
        let flushed = rt.console.flush();
        rt.check(flushed)
    });
}

// ---- every statement ------------------------------------------------------------

/// The height of the stack of temporary strings.
#[no_mangle]
pub extern "C" fn rt_mark() -> i32 {
    with(|rt| rt.strings.mark() as i32)
}

/// Called before each operation: frees the temporary strings of the one
/// before, notes the line for error messages, and counts steps.
#[no_mangle]
pub extern "C" fn rt_op(mark: i32, line: i32) {
    with(|rt| {
        rt.strings.release(mark as usize);
        rt.line = u32::try_from(line).ok();
        rt.steps += 1;
        if rt.step_limit.is_some_and(|limit| rt.steps > limit) {
            rt.fail("Step limit exceeded");
        }
    });
}

/// Stop with the runtime error whose message is the string `message`.
#[no_mangle]
pub extern "C" fn rt_fail(message: i32) {
    with(|rt| {
        let message = rt.strings.get(message).to_string();
        rt.fail(&message)
    });
}

#[no_mangle]
pub extern "C" fn rt_type_mismatch() {
    with(|rt| rt.fail("Type mismatch"));
}

/// END or STOP.
#[no_mangle]
pub extern "C" fn rt_end() {
    with(|rt| rt.exit(0));
}

// ---- strings --------------------------------------------------------------------

/// A copy of string variable `slot`.
#[no_mangle]
pub extern "C" fn rt_sget(slot: i32) -> i32 {
    with(|rt| {
        let text = rt.string_vars[slot as usize].clone();
        rt.strings.temp(text)
    })
}

#[no_mangle]
pub extern "C" fn rt_sset(slot: i32, value: i32) {
    with(|rt| rt.string_vars[slot as usize] = rt.strings.get(value).to_string());
}

#[no_mangle]
pub extern "C" fn rt_concat(left: i32, right: i32) -> i32 {
    with(|rt| {
        let joined = format!("{}{}", rt.strings.get(left), rt.strings.get(right));
        rt.strings.temp(joined)
    })
}

/// Compare two strings: -1, 0 or 1.
#[no_mangle]
pub extern "C" fn rt_scmp(left: i32, right: i32) -> i32 {
    with(|rt| rt.strings.get(left).cmp(rt.strings.get(right)) as i32)
}

// ---- numbers --------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        with(|rt| rt.fail("Division by zero"));
    }
    a / b
}

#[no_mangle]
pub extern "C" fn rt_pow(a: f64, b: f64) -> f64 {
    with(|rt| match rt.dialect.power(a, b) {
        Some(n) => n,
        None => rt.fail("Illegal function call: ^"),
    })
}

/// A minus written directly before a power, `-A ^ B` (see
/// `Dialect::leading_minus`), given the value of `A ^ B`.
#[no_mangle]
pub extern "C" fn rt_leading_minus(v: f64) -> f64 {
    with(|rt| rt.dialect.leading_minus(v))
}

#[no_mangle]
pub extern "C" fn rt_and(a: f64, b: f64) -> f64 {
    ((a as i64) & (b as i64)) as f64
}

#[no_mangle]
pub extern "C" fn rt_or(a: f64, b: f64) -> f64 {
    ((a as i64) | (b as i64)) as f64
}

#[no_mangle]
pub extern "C" fn rt_not(a: f64) -> f64 {
    !(a as i64) as f64
}

/// Rounding for `%` variables.
#[no_mangle]
pub extern "C" fn rt_round(a: f64) -> f64 {
    a.round()
}

// ---- arrays ---------------------------------------------------------------------

/// One subscript (or DIM bound) of the array access being built up.
#[no_mangle]
pub extern "C" fn rt_idx(value: f64) {
    with(|rt| {
        let index = rt.check(subscript(value));
        rt.indices.push(index);
    });
}

#[no_mangle]
pub extern "C" fn rt_dim(array: i32, count: i32) {
    with(|rt| {
        let bounds = rt.take_indices(count);
        let done = rt.arrays.dim(&rt.array_names[array as usize], &bounds);
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_aget_num(array: i32, count: i32) -> f64 {
    with(|rt| match rt.element(array, count) {
        Value::Num(n) => n,
        Value::Str(_) => rt.fail("Type mismatch"),
    })
}

#[no_mangle]
pub extern "C" fn rt_aget_str(array: i32, count: i32) -> i32 {
    with(|rt| match rt.element(array, count) {
        Value::Str(text) => rt.strings.temp(text),
        Value::Num(_) => rt.fail("Type mismatch"),
    })
}

#[no_mangle]
pub extern "C" fn rt_aset_num(value: f64, array: i32, count: i32) {
    with(|rt| rt.set_element(array, count, Value::Num(value)));
}

#[no_mangle]
pub extern "C" fn rt_aset_str(value: i32, array: i32, count: i32) {
    with(|rt| {
        let value = Value::Str(rt.strings.get(value).to_string());
        rt.set_element(array, count, value)
    });
}

// ---- built-in functions -----------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_arg_num(value: f64) {
    with(|rt| rt.args.push(Value::Num(value)));
}

#[no_mangle]
pub extern "C" fn rt_arg_str(value: i32) {
    with(|rt| {
        let value = Value::Str(rt.strings.get(value).to_string());
        rt.args.push(value)
    });
}

fn call_builtin(rt: &mut Runtime, builtin: i32, count: i32) -> Value {
    let builtin =
        Builtin::from_name(Builtin::NAMES[builtin as usize]).expect("a built-in function");
    let args = rt.take_args(count);
    let result = builtins::call(builtin, &args, &mut rt.rnd, rt.dialect);
    rt.check(result)
}

/// Call built-in function number `builtin` (its place in `Builtin::NAMES`)
/// with the last `count` arguments passed.
#[no_mangle]
pub extern "C" fn rt_call_num(builtin: i32, count: i32) -> f64 {
    with(|rt| match call_builtin(rt, builtin, count) {
        Value::Num(n) => n,
        Value::Str(_) => rt.fail("Type mismatch"),
    })
}

#[no_mangle]
pub extern "C" fn rt_call_str(builtin: i32, count: i32) -> i32 {
    with(|rt| match call_builtin(rt, builtin, count) {
        Value::Str(text) => rt.strings.temp(text),
        Value::Num(_) => rt.fail("Type mismatch"),
    })
}

// ---- PRINT ------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_print_num(value: f64) {
    with(|rt| {
        let done = rt.console.print_value(&Value::Num(value));
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_print_str(value: i32) {
    with(|rt| {
        let done = rt.console.write(rt.strings.get(value));
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_tab(column: f64) {
    with(|rt| {
        let done = rt.console.tab(column);
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_spc(count: f64) {
    with(|rt| {
        let done = rt.console.spc(count);
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_zone() {
    with(|rt| {
        let done = rt.console.next_zone();
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_packed() {
    with(|rt| {
        let done = rt.console.packed();
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_newline() {
    with(|rt| {
        let done = rt.console.newline();
        rt.check(done)
    });
}

/// PRINT USING: `format` is the picture, and the last `count` arguments
/// passed (as for a built-in function) are the values.
#[no_mangle]
pub extern "C" fn rt_using(format: i32, count: i32, newline: i32) {
    with(|rt| {
        let values = rt.take_args(count);
        let formatted = format_using(rt.strings.get(format), &values);
        let mut text = rt.check(formatted);
        if newline != 0 {
            text.push('\n');
        }
        let done = rt.console.write(&text);
        rt.check(done)
    });
}

// ---- INPUT ------------------------------------------------------------------------

/// Say what the INPUT about to happen wants next: a string or a number.
#[no_mangle]
pub extern "C" fn rt_input_want(string: i32) {
    with(|rt| rt.input_kinds.push(string != 0));
}

/// Do the INPUT; the answers are then fetched with `rt_input_num` and
/// `rt_input_str`.
#[no_mangle]
pub extern "C" fn rt_input(prompt: i32, has_prompt: i32, question_mark: i32) {
    with(|rt| {
        let prompt = (has_prompt != 0).then(|| rt.strings.get(prompt).to_string());
        let kinds = std::mem::take(&mut rt.input_kinds);
        let answers = rt
            .console
            .input(prompt.as_deref(), question_mark != 0, &kinds);
        rt.input_values = rt.check(answers);
    });
}

#[no_mangle]
pub extern "C" fn rt_input_num(index: i32) -> f64 {
    with(|rt| match &rt.input_values[index as usize] {
        Value::Num(n) => *n,
        Value::Str(_) => rt.fail("Type mismatch"),
    })
}

#[no_mangle]
pub extern "C" fn rt_input_str(index: i32) -> i32 {
    with(|rt| match rt.input_values[index as usize].clone() {
        Value::Str(text) => rt.strings.temp(text),
        Value::Num(_) => rt.fail("Type mismatch"),
    })
}

#[no_mangle]
pub extern "C" fn rt_line_input(prompt: i32, has_prompt: i32) -> i32 {
    with(|rt| {
        let prompt = (has_prompt != 0).then(|| rt.strings.get(prompt).to_string());
        let line = rt.console.line_input(prompt.as_deref());
        let line = rt.check(line);
        rt.strings.temp(line)
    })
}

// ---- READ, RESTORE, RANDOMIZE -------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_read_num() -> f64 {
    with(|rt| {
        if rt.dialect.read_past_data_ends() && rt.data.exhausted() {
            rt.exit(0);
        }
        let item = rt.data.read(false);
        match rt.check(item) {
            Value::Num(n) => n,
            Value::Str(_) => rt.fail("Type mismatch"),
        }
    })
}

#[no_mangle]
pub extern "C" fn rt_read_str() -> i32 {
    with(|rt| {
        if rt.dialect.read_past_data_ends() && rt.data.exhausted() {
            rt.exit(0);
        }
        let item = rt.data.read(true);
        match rt.check(item) {
            Value::Str(text) => rt.strings.temp(text),
            Value::Num(_) => rt.fail("Type mismatch"),
        }
    })
}

/// RESTORE; `line` is -1 for "from the start".
#[no_mangle]
pub extern "C" fn rt_restore(line: i32) {
    with(|rt| rt.data.restore(u32::try_from(line).ok()));
}

#[no_mangle]
pub extern "C" fn rt_randomize(has_seed: i32, seed: f64) {
    with(|rt| rt.rnd.randomize((has_seed != 0).then_some(seed)));
}

// ---- FOR, NEXT, GOSUB, RETURN, ON -----------------------------------------------------

/// FOR: set the variable and open the loop, whose body starts at operation
/// `body`. Returns 1 if the range is empty and the loop must be skipped.
#[no_mangle]
pub extern "C" fn rt_for(slot: i32, start: f64, limit: f64, step: f64, body: i32) -> i32 {
    with(|rt| {
        let slot = slot as usize;
        rt.numbers[slot] = start;
        // re-entering a loop that is still open replaces it
        let base = rt.for_base();
        if let Some(at) = rt.for_stack[base..]
            .iter()
            .rposition(|frame| frame.slot == slot)
        {
            rt.for_stack.truncate(base + at);
        }
        if (step > 0.0 && start > limit) || (step < 0.0 && start < limit) {
            return 1;
        }
        rt.for_stack.push(ForFrame {
            slot,
            limit,
            step,
            body,
        });
        0
    })
}

/// NEXT (`slot` is -1 when no variable is named). Returns the operation to
/// continue the loop at, or -1 when the loop has finished.
#[no_mangle]
pub extern "C" fn rt_next(slot: i32) -> i32 {
    with(|rt| {
        let base = rt.for_base();
        // NEXT with a variable closes any inner loops that were jumped out of
        while rt.for_stack.len() > base {
            let top = rt.for_stack.last().expect("non-empty");
            if slot < 0 || top.slot == slot as usize {
                break;
            }
            rt.for_stack.pop();
        }
        if rt.for_stack.len() <= base {
            rt.fail("NEXT without FOR");
        }
        let ForFrame {
            slot,
            limit,
            step,
            body,
        } = *rt.for_stack.last().expect("non-empty");
        let value = rt.numbers[slot] + step;
        rt.numbers[slot] = value;
        if (step >= 0.0 && value <= limit) || (step < 0.0 && value >= limit) {
            body
        } else {
            rt.for_stack.pop();
            -1
        }
    })
}

/// GOSUB: remember that RETURN goes back to operation `ret`.
#[no_mangle]
pub extern "C" fn rt_gosub(ret: i32) {
    with(|rt| {
        if rt.gosub_stack.len() >= MAX_GOSUB_DEPTH {
            rt.fail("Out of memory: GOSUB nested too deeply");
        }
        let for_base = rt.for_stack.len();
        rt.gosub_stack.push(GosubFrame { ret, for_base });
    });
}

/// RETURN: the operation to go back to.
#[no_mangle]
pub extern "C" fn rt_return() -> i32 {
    with(|rt| {
        if rt.gosub_stack.len() <= rt.fn_gosub_base {
            rt.fail("RETURN without GOSUB");
        }
        let frame = rt.gosub_stack.pop().expect("checked above");
        rt.for_stack.truncate(frame.for_base);
        frame.ret
    })
}

/// ON x GOTO/GOSUB with `count` targets: which one (from 1), or 0 for none.
#[no_mangle]
pub extern "C" fn rt_on(value: f64, count: i32) -> i32 {
    let n = value.round();
    if n >= 1.0 && n <= count as f64 {
        n as i32
    } else {
        0
    }
}

// ---- user functions ---------------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_fn_enter() {
    with(|rt| {
        if rt.fn_depth >= MAX_FN_DEPTH {
            rt.fail("Out of memory: function calls nested too deeply");
        }
        rt.fn_depth += 1;
    });
}

#[no_mangle]
pub extern "C" fn rt_fn_exit() {
    with(|rt| rt.fn_depth -= 1);
}

/// Entering the body of a multi-line function: GOSUBs and loops opened
/// outside it are out of reach until it ends.
#[no_mangle]
pub extern "C" fn rt_block_enter() {
    with(|rt| {
        rt.saved_bases
            .push((rt.fn_gosub_base, rt.fn_for_base, rt.line));
        rt.fn_gosub_base = rt.gosub_stack.len();
        rt.fn_for_base = rt.for_stack.len();
    });
}

#[no_mangle]
pub extern "C" fn rt_block_exit() {
    with(|rt| {
        rt.gosub_stack.truncate(rt.fn_gosub_base);
        rt.for_stack.truncate(rt.fn_for_base);
        let (gosub_base, for_base, line) = rt.saved_bases.pop().expect("a function is running");
        (rt.fn_gosub_base, rt.fn_for_base, rt.line) = (gosub_base, for_base, line);
    });
}

// ---- MAT ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn rt_mat_read(array: i32, count: i32) {
    with(|rt| {
        let dims = rt.take_indices(count);
        let done = mat::read(
            &mut rt.arrays,
            &mut rt.data,
            &rt.array_names[array as usize],
            &dims,
        );
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_mat_input(array: i32, count: i32) {
    with(|rt| {
        let dims = rt.take_indices(count);
        let done = mat::input(
            &mut rt.arrays,
            &mut rt.console,
            &rt.array_names[array as usize],
            &dims,
        );
        rt.check(done)
    });
}

#[no_mangle]
pub extern "C" fn rt_mat_print(array: i32, packed: i32) {
    with(|rt| {
        let done = mat::print(
            &rt.arrays,
            &mut rt.console,
            &rt.array_names[array as usize],
            packed != 0,
        );
        rt.check(done)
    });
}

/// `MAT target = ...`. `op` selects the operation (see `MAT_*` in the
/// compiler); `a` and `b` are its matrix operands, `factor` the scale
/// factor, and `count` the number of dimensions passed with `rt_idx`.
#[no_mangle]
pub extern "C" fn rt_mat_assign(target: i32, op: i32, a: i32, b: i32, factor: f64, count: i32) {
    with(|rt| {
        let dims = rt.take_indices(count);
        let name = |id: i32| rt.array_names[id as usize].as_str();
        let op = match op {
            0 => MatOp::Copy(name(a)),
            1 => MatOp::Add(name(a), name(b)),
            2 => MatOp::Sub(name(a), name(b)),
            3 => MatOp::Mul(name(a), name(b)),
            4 => MatOp::Scale(factor, name(a)),
            5 => MatOp::Zer(&dims),
            6 => MatOp::Con(&dims),
            7 => MatOp::Idn(&dims),
            8 => MatOp::Trn(name(a)),
            _ => MatOp::Inv(name(a)),
        };
        let done = mat::assign(&mut rt.arrays, name(target), op);
        rt.check(done)
    });
}
