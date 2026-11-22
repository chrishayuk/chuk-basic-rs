//! The state of a compilation, and the top level of code generation.

use std::collections::HashMap;
use std::rc::Rc;

use super::code::{Code, Op as I};
use super::module::{Module, ValType};
use crate::interp::lower::{Func, Lowered, Op};

/// What an expression leaves on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Ty {
    /// A number: an `f64`.
    Num,
    /// A string: an `i32` handle.
    Str,
}

impl Ty {
    /// The type of a variable or function, which its name decides.
    pub(super) fn of(name: &str) -> Ty {
        if name.ends_with('$') {
            Ty::Str
        } else {
            Ty::Num
        }
    }

    pub(super) fn val(self) -> ValType {
        match self {
            Ty::Num => ValType::F64,
            Ty::Str => ValType::I32,
        }
    }
}

macro_rules! runtime_functions {
    ($($name:ident,)*) => {
        /// The indices of the runtime's functions.
        pub(super) struct Runtime { $(pub(super) $name: u32,)* }

        impl Runtime {
            fn find(module: &Module) -> Result<Runtime, String> {
                Ok(Runtime {
                    $($name: module.exported_function(stringify!($name)).ok_or_else(|| {
                        format!("The runtime module has no function {}; rebuild it", stringify!($name))
                    })?,)*
                })
            }
        }
    };
}

runtime_functions! {
    rt_init, rt_bytes, rt_add_literal, rt_add_data, rt_data_line, rt_add_array, rt_num_base, rt_finish,
    rt_mark, rt_op, rt_fail, rt_type_mismatch, rt_end,
    rt_sget, rt_sset, rt_concat, rt_scmp,
    rt_div, rt_pow, rt_leading_minus, rt_and, rt_or, rt_not, rt_round,
    rt_idx, rt_dim, rt_aget_num, rt_aget_str, rt_aset_num, rt_aset_str,
    rt_arg_num, rt_arg_str, rt_call_num, rt_call_str,
    rt_print_num, rt_print_str, rt_tab, rt_spc, rt_zone, rt_newline, rt_using,
    rt_input_want, rt_input, rt_input_num, rt_input_str, rt_line_input,
    rt_read_num, rt_read_str, rt_restore, rt_randomize,
    rt_for, rt_next, rt_gosub, rt_return, rt_on,
    rt_fn_enter, rt_fn_exit, rt_block_enter, rt_block_exit,
    rt_mat_read, rt_mat_input, rt_mat_print, rt_mat_assign,
}

/// Numbers handed out to names in the order they are first seen.
#[derive(Default)]
pub(super) struct Slots {
    by_name: HashMap<String, u32>,
    names: Vec<String>,
}

impl Slots {
    pub(super) fn get(&mut self, name: &str) -> u32 {
        if let Some(&slot) = self.by_name.get(name) {
            return slot;
        }
        self.names.push(name.to_string());
        self.by_name
            .insert(name.to_string(), self.names.len() as u32 - 1);
        self.names.len() as u32 - 1
    }
}

/// A user function: its WebAssembly function, and its definition.
pub(super) struct UserFn {
    pub(super) index: u32,
    pub(super) def: Rc<Func>,
}

pub(super) struct Gen {
    pub(super) module: Module,
    pub(super) rt: Runtime,
    pub(super) lowered: Lowered,
    pub(super) functions: HashMap<String, UserFn>,
    /// The function holding the whole program: `run(start, in_function)`.
    pub(super) run: u32,
    pub(super) numbers: Slots,
    pub(super) strings: Slots,
    pub(super) arrays: Slots,
    /// String literals; the handle of a literal is its number plus one.
    pub(super) literals: Slots,
}

/// The function being generated.
pub(super) struct Body {
    pub(super) code: Code,
    /// Local holding the address of the numeric variables.
    pub(super) numbers_at: u32,
    /// An `i32` local for short-lived values.
    pub(super) scratch: u32,
}

impl Body {
    /// Start a function with `params` parameters; emits the code that
    /// fetches the address of the variables.
    pub(super) fn new(params: u32, rt: &Runtime) -> Body {
        let mut code = Code::new(params);
        let numbers_at = code.local(ValType::I32);
        let scratch = code.local(ValType::I32);
        code.call(rt.rt_num_base);
        code.local_set(numbers_at);
        Body {
            code,
            numbers_at,
            scratch,
        }
    }
}

/// Locals of `run`: its two parameters, then the base of its temporaries.
const PC: u32 = 0;
const IN_FUNCTION: u32 = 1;

impl Gen {
    pub(super) fn new(lowered: Lowered, runtime: &[u8]) -> Result<Gen, String> {
        let mut module = Module::parse(runtime)?;
        let rt = Runtime::find(&module)?;
        let run = module.declare(&[ValType::I32, ValType::I32], &[]);

        // every user function is declared before any code is generated, so
        // that they can call each other
        let mut names: Vec<&String> = lowered.funcs.keys().collect();
        names.sort();
        let mut functions = HashMap::new();
        for name in names {
            let def = Rc::clone(&lowered.funcs[name]);
            let params: Vec<ValType> = def.params.iter().map(|p| Ty::of(p).val()).collect();
            let index = module.declare(&params, &[Ty::of(name).val()]);
            functions.insert(name.clone(), UserFn { index, def });
        }

        Ok(Gen {
            module,
            rt,
            lowered,
            functions,
            run,
            numbers: Slots::default(),
            strings: Slots::default(),
            arrays: Slots::default(),
            literals: Slots::default(),
        })
    }

    /// Generate everything and return the finished module.
    pub(super) fn finish(mut self) -> Result<Vec<u8>, String> {
        self.program();
        let mut names: Vec<String> = self.functions.keys().cloned().collect();
        names.sort();
        for name in names {
            self.user_function(&name);
        }
        self.start();

        // the runtime's functions were exported for the compiler's benefit
        self.module.retain_exports(|name| !name.starts_with("rt_"));
        Ok(self.module.finish())
    }

    /// The handle of the string literal `text`.
    pub(super) fn literal(&mut self, text: &str) -> i32 {
        self.literals.get(text) as i32 + 1
    }

    /// `run`: a loop around a jump table, with one block per operation.
    fn program(&mut self) {
        let ops = std::mem::take(&mut self.lowered.ops);
        let count = ops.len() as u32;
        let mut body = Body::new(2, &self.rt);
        let mark = body.code.local(ValType::I32);
        body.code.call(self.rt.rt_mark);
        body.code.local_set(mark);

        body.code.loop_();
        body.code.block(); // leaving this block ends the program
        for _ in 0..count {
            body.code.block();
        }
        body.code.local_get(PC);
        body.code.br_table(0..count, count);
        for (index, op) in ops.iter().enumerate() {
            body.code.op(I::End); // the jump table lands here for `index`
            let line = self.lowered.op_line[index].map_or(-1, |line| line as i32);
            body.code.local_get(mark);
            body.code.i32_const(line);
            body.code.call(self.rt.rt_op);
            self.op(&mut body, &ops, index, op);
        }
        body.code.op(I::End);
        body.code.op(I::Return);
        body.code.op(I::End); // of the loop

        let (locals, instructions) = body.code.finish();
        self.module.define(self.run, &locals, &instructions);
        self.lowered.ops = ops;
    }

    /// Jump to operation `target`: set the program counter and go round.
    pub(super) fn jump(&mut self, body: &mut Body, target: usize) {
        body.code.i32_const(target as i32);
        self.jump_to_stack(body);
    }

    /// Jump to the operation whose number is on the stack.
    pub(super) fn jump_to_stack(&mut self, body: &mut Body) {
        body.code.local_set(PC);
        // the loop is the outermost construct of `run`
        body.code.br(body.code.depth() - 1);
    }

    /// Whether `run` was called for the body of a multi-line function.
    pub(super) fn in_function(&mut self, body: &mut Body) {
        body.code.local_get(IN_FUNCTION);
    }

    /// Hand `text` to the runtime, eight bytes at a time.
    fn send_text(&self, code: &mut Code, text: &str) {
        for chunk in text.as_bytes().chunks(8) {
            let mut bytes = [0u8; 8];
            bytes[..chunk.len()].copy_from_slice(chunk);
            code.i64_const(i64::from_le_bytes(bytes));
            code.i32_const(chunk.len() as i32);
            code.call(self.rt.rt_bytes);
        }
    }

    /// `_start`: set the runtime up, tell it the program's literals, DATA
    /// and array names, and run the program.
    fn start(&mut self) {
        let mut code = Code::new(0);
        code.i32_const(self.numbers.names.len() as i32);
        code.i32_const(self.strings.names.len() as i32);
        code.call(self.rt.rt_init);

        for literal in &self.literals.names {
            self.send_text(&mut code, literal);
            code.call(self.rt.rt_add_literal);
            code.op(I::Drop);
        }
        for name in &self.arrays.names {
            self.send_text(&mut code, name);
            code.call(self.rt.rt_add_array);
        }
        let (items, lines) = self.lowered.data.contents();
        let mut lines = lines.iter().peekable();
        for index in 0..=items.len() {
            while let Some((number, _)) = lines.next_if(|(_, at)| *at == index) {
                code.i32_const(*number as i32);
                code.call(self.rt.rt_data_line);
            }
            if let Some(item) = items.get(index) {
                self.send_text(&mut code, &item.text);
                code.i32_const(i32::from(item.quoted));
                code.call(self.rt.rt_add_data);
            }
        }

        code.i32_const(0);
        code.i32_const(0);
        code.call(self.run);
        code.call(self.rt.rt_finish);

        let start = self.module.declare(&[], &[]);
        let (locals, instructions) = code.finish();
        self.module.define(start, &locals, &instructions);
        self.module.export("_start", start);
    }

    /// The operation after the NEXT that closes the FOR at `index`, if
    /// there is one: where an empty loop skips to.
    pub(super) fn after_matching_next(ops: &[Op], index: usize) -> Option<usize> {
        let mut depth = 0;
        for (at, op) in ops.iter().enumerate().skip(index + 1) {
            match op {
                Op::Stmt(crate::ast::Stmt::For { .. }) => depth += 1,
                Op::Next(_) if depth == 0 => return Some(at + 1),
                Op::Next(_) => depth -= 1,
                _ => {}
            }
        }
        None
    }
}
