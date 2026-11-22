//! Writes the instructions of one function.

use super::leb;
use super::module::ValType;

/// The instructions used by the compiler, by opcode.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Op {
    Unreachable = 0x00,
    End = 0x0B,
    Return = 0x0F,
    Drop = 0x1A,
    Select = 0x1B,
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32GtS = 0x4A,
    I32LeS = 0x4C,
    I32GeS = 0x4E,
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,
    F64Abs = 0x99,
    F64Neg = 0x9A,
    F64Trunc = 0x9D,
    F64Add = 0xA0,
    F64Sub = 0xA1,
    F64Mul = 0xA2,
}

const EMPTY_BLOCK: u8 = 0x40;

/// A function body under construction.
pub struct Code {
    bytes: Vec<u8>,
    /// Locals beyond the parameters.
    locals: Vec<ValType>,
    params: u32,
    /// How many blocks, loops and ifs are open: `br` counts outwards from
    /// the innermost.
    depth: u32,
}

impl Code {
    pub fn new(params: u32) -> Self {
        Code {
            bytes: Vec::new(),
            locals: Vec::new(),
            params,
            depth: 0,
        }
    }

    pub fn finish(self) -> (Vec<ValType>, Vec<u8>) {
        debug_assert_eq!(self.depth, 0, "unclosed block");
        (self.locals, self.bytes)
    }

    /// A new local variable; returns its index.
    pub fn local(&mut self, ty: ValType) -> u32 {
        self.locals.push(ty);
        self.params + self.locals.len() as u32 - 1
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn op(&mut self, op: Op) {
        if matches!(op, Op::End) {
            self.depth -= 1;
        }
        self.bytes.push(op as u8);
    }

    fn indexed(&mut self, opcode: u8, index: u32) {
        self.bytes.push(opcode);
        leb::unsigned(&mut self.bytes, u64::from(index));
    }

    pub fn i32_const(&mut self, value: i32) {
        self.bytes.push(0x41);
        leb::signed(&mut self.bytes, i64::from(value));
    }

    pub fn i64_const(&mut self, value: i64) {
        self.bytes.push(0x42);
        leb::signed(&mut self.bytes, value);
    }

    pub fn f64_const(&mut self, value: f64) {
        self.bytes.push(0x44);
        self.bytes.extend(value.to_le_bytes());
    }

    pub fn local_get(&mut self, local: u32) {
        self.indexed(0x20, local);
    }

    pub fn local_set(&mut self, local: u32) {
        self.indexed(0x21, local);
    }

    pub fn local_tee(&mut self, local: u32) {
        self.indexed(0x22, local);
    }

    pub fn call(&mut self, function: u32) {
        self.indexed(0x10, function);
    }

    /// Load the `f64` at the address on the stack plus `offset`.
    pub fn f64_load(&mut self, offset: u32) {
        self.indexed(0x2B, 3); // aligned to 8 bytes
        leb::unsigned(&mut self.bytes, u64::from(offset));
    }

    pub fn f64_store(&mut self, offset: u32) {
        self.indexed(0x39, 3);
        leb::unsigned(&mut self.bytes, u64::from(offset));
    }

    pub fn block(&mut self) {
        self.bytes.extend([0x02, EMPTY_BLOCK]);
        self.depth += 1;
    }

    pub fn loop_(&mut self) {
        self.bytes.extend([0x03, EMPTY_BLOCK]);
        self.depth += 1;
    }

    /// `if` on the i32 on the stack; closed with [`Op::End`].
    pub fn if_(&mut self) {
        self.bytes.extend([0x04, EMPTY_BLOCK]);
        self.depth += 1;
    }

    /// Branch to the block `outwards` levels out from the innermost.
    pub fn br(&mut self, outwards: u32) {
        self.indexed(0x0C, outwards);
    }

    /// Branch to `labels[n]` for the i32 `n` on the stack, or to `default`.
    pub fn br_table(&mut self, labels: impl ExactSizeIterator<Item = u32>, default: u32) {
        self.bytes.push(0x0E);
        leb::unsigned(&mut self.bytes, labels.len() as u64);
        for label in labels {
            leb::unsigned(&mut self.bytes, u64::from(label));
        }
        leb::unsigned(&mut self.bytes, u64::from(default));
    }
}
