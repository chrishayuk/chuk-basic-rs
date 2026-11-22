//! Reads a WebAssembly module and adds functions to it.
//!
//! This is the "linker": the module read is the prebuilt runtime, and the
//! functions added are the compiled program, which calls the runtime's
//! exported functions by index.

use std::collections::HashMap;

use super::leb::{self, Reader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValType {
    I32 = 0x7F,
    F64 = 0x7C,
}

const SECTION_CUSTOM: u8 = 0;
const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;

const KIND_FUNCTION: u8 = 0;

struct Export {
    name: String,
    kind: u8,
    index: u32,
}

/// A section that is a count followed by that many entries, to which more
/// entries can be appended without decoding the existing ones.
#[derive(Default)]
struct Entries {
    count: u32,
    bytes: Vec<u8>,
}

impl Entries {
    fn parse(payload: &[u8]) -> Option<Entries> {
        let mut reader = Reader::new(payload);
        let count = reader.unsigned()?;
        Some(Entries {
            count,
            bytes: reader.rest().to_vec(),
        })
    }

    fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        leb::unsigned(&mut out, u64::from(self.count));
        out.extend_from_slice(&self.bytes);
        out
    }
}

pub struct Module {
    /// The sections that are passed through untouched, in order.
    other: Vec<(u8, Vec<u8>)>,
    types: Entries,
    functions: Entries,
    code: Entries,
    exports: Vec<Export>,
    imported_functions: u32,
    /// Functions added: their type, and their body once it is known.
    added: Vec<(u32, Option<Vec<u8>>)>,
    /// Types added, to reuse one for each distinct signature.
    added_types: HashMap<(Vec<ValType>, Vec<ValType>), u32>,
}

impl Module {
    /// Read a module. Custom sections (names, producers) are dropped.
    pub fn parse(bytes: &[u8]) -> Result<Module, String> {
        let malformed = || "The runtime module is not a valid WebAssembly module".to_string();
        let mut reader = Reader::new(bytes);
        if reader.take(8) != Some(b"\0asm\x01\0\0\0") {
            return Err(malformed());
        }
        let mut module = Module {
            other: Vec::new(),
            types: Entries::default(),
            functions: Entries::default(),
            code: Entries::default(),
            exports: Vec::new(),
            imported_functions: 0,
            added: Vec::new(),
            added_types: HashMap::new(),
        };
        while !reader.at_end() {
            let id = reader.byte().ok_or_else(malformed)?;
            let size = reader.unsigned().ok_or_else(malformed)? as usize;
            let payload = reader.take(size).ok_or_else(malformed)?;
            match id {
                SECTION_CUSTOM => {}
                SECTION_TYPE => module.types = Entries::parse(payload).ok_or_else(malformed)?,
                SECTION_FUNCTION => {
                    module.functions = Entries::parse(payload).ok_or_else(malformed)?
                }
                SECTION_CODE => module.code = Entries::parse(payload).ok_or_else(malformed)?,
                SECTION_EXPORT => module.exports = parse_exports(payload).ok_or_else(malformed)?,
                SECTION_IMPORT => {
                    module.imported_functions =
                        count_imported_functions(payload).ok_or_else(malformed)?;
                    module.other.push((id, payload.to_vec()));
                }
                _ => module.other.push((id, payload.to_vec())),
            }
        }
        Ok(module)
    }

    /// The index of the exported function called `name`.
    pub fn exported_function(&self, name: &str) -> Option<u32> {
        self.exports
            .iter()
            .find(|e| e.kind == KIND_FUNCTION && e.name == name)
            .map(|e| e.index)
    }

    fn type_index(&mut self, params: &[ValType], results: &[ValType]) -> u32 {
        let key = (params.to_vec(), results.to_vec());
        if let Some(&index) = self.added_types.get(&key) {
            return index;
        }
        let index = self.types.count;
        self.types.count += 1;
        self.types.bytes.push(0x60); // a function type
        for list in [params, results] {
            leb::unsigned(&mut self.types.bytes, list.len() as u64);
            self.types.bytes.extend(list.iter().map(|&ty| ty as u8));
        }
        self.added_types.insert(key, index);
        index
    }

    /// Reserve a new function with this signature and return its index, so
    /// that calls to it can be generated before its body is.
    pub fn declare(&mut self, params: &[ValType], results: &[ValType]) -> u32 {
        let type_index = self.type_index(params, results);
        self.added.push((type_index, None));
        self.imported_functions + self.functions.count + self.added.len() as u32 - 1
    }

    /// Supply the body of a declared function: its extra locals, and its
    /// instructions (without the final `end`).
    pub fn define(&mut self, function: u32, locals: &[ValType], instructions: &[u8]) {
        let mut body = Vec::new();
        leb::unsigned(&mut body, locals.len() as u64);
        for &local in locals {
            body.extend([1, local as u8]); // one local of this type
        }
        body.extend_from_slice(instructions);
        body.push(0x0B); // end
        let slot = (function - self.imported_functions - self.functions.count) as usize;
        self.added[slot].1 = Some(body);
    }

    pub fn export(&mut self, name: &str, function: u32) {
        self.exports.retain(|e| e.name != name);
        self.exports.push(Export {
            name: name.to_string(),
            kind: KIND_FUNCTION,
            index: function,
        });
    }

    /// Keep only the exports for which `keep` says yes.
    pub fn retain_exports(&mut self, keep: impl Fn(&str) -> bool) {
        self.exports.retain(|e| keep(&e.name));
    }

    /// The finished module.
    pub fn finish(mut self) -> Vec<u8> {
        for (type_index, body) in std::mem::take(&mut self.added) {
            let body = body.expect("every declared function is defined");
            self.functions.count += 1;
            leb::unsigned(&mut self.functions.bytes, u64::from(type_index));
            self.code.count += 1;
            leb::unsigned(&mut self.code.bytes, body.len() as u64);
            self.code.bytes.extend(body);
        }

        let mut exports = Vec::new();
        leb::unsigned(&mut exports, self.exports.len() as u64);
        for export in &self.exports {
            leb::unsigned(&mut exports, export.name.len() as u64);
            exports.extend(export.name.bytes());
            exports.push(export.kind);
            leb::unsigned(&mut exports, u64::from(export.index));
        }

        let mut sections = self.other;
        sections.push((SECTION_TYPE, self.types.payload()));
        sections.push((SECTION_FUNCTION, self.functions.payload()));
        sections.push((SECTION_EXPORT, exports));
        sections.push((SECTION_CODE, self.code.payload()));
        sections.sort_by_key(|(id, _)| section_order(*id)); // stable: keeps the rest in place

        let mut out = b"\0asm\x01\0\0\0".to_vec();
        for (id, payload) in sections {
            out.push(id);
            leb::unsigned(&mut out, payload.len() as u64);
            out.extend(payload);
        }
        out
    }
}

/// Sections must appear in a fixed order, which is not quite the order of
/// their numbers: data count (12) comes before code (10).
fn section_order(id: u8) -> u8 {
    match id {
        12 => 10,
        10 => 11,
        11 => 12,
        other => other,
    }
}

fn parse_exports(payload: &[u8]) -> Option<Vec<Export>> {
    let mut reader = Reader::new(payload);
    let count = reader.unsigned()?;
    (0..count)
        .map(|_| {
            Some(Export {
                name: reader.name()?.to_string(),
                kind: reader.byte()?,
                index: reader.unsigned()?,
            })
        })
        .collect()
}

fn count_imported_functions(payload: &[u8]) -> Option<u32> {
    let mut reader = Reader::new(payload);
    let mut functions = 0;
    for _ in 0..reader.unsigned()? {
        reader.name()?; // module
        reader.name()?; // field
        match reader.byte()? {
            KIND_FUNCTION => {
                reader.unsigned()?;
                functions += 1;
            }
            // a table: element type, then limits
            1 => {
                reader.byte()?;
                skip_limits(&mut reader)?;
            }
            // a memory: limits
            2 => skip_limits(&mut reader)?,
            // a global: type and mutability
            3 => {
                reader.take(2)?;
            }
            _ => return None,
        }
    }
    Some(functions)
}

fn skip_limits(reader: &mut Reader) -> Option<()> {
    let flags = reader.byte()?;
    reader.unsigned()?;
    if flags & 1 != 0 {
        reader.unsigned()?;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (module
    ///   (import "env" "log" (func (param i32)))
    ///   (func (export "one") (result i32) i32.const 1))
    const SMALL: &[u8] = &[
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, // header
        0x01, 0x09, 0x02, 0x60, 0x01, 0x7F, 0x00, 0x60, 0x00, 0x01, 0x7F, // types
        0x02, 0x0B, 0x01, 0x03, b'e', b'n', b'v', 0x03, b'l', b'o', b'g', 0x00,
        0x00, // import
        0x03, 0x02, 0x01, 0x01, // functions
        0x07, 0x07, 0x01, 0x03, b'o', b'n', b'e', 0x00, 0x01, // exports
        0x0A, 0x06, 0x01, 0x04, 0x00, 0x41, 0x01, 0x0B, // code
    ];

    #[test]
    fn finds_exports_and_numbers_new_functions_after_the_old() {
        let mut module = Module::parse(SMALL).unwrap();
        assert_eq!(module.exported_function("one"), Some(1));
        assert_eq!(module.exported_function("two"), None);
        let first = module.declare(&[], &[ValType::I32]);
        let second = module.declare(&[ValType::F64], &[]);
        assert_eq!((first, second), (2, 3));
    }

    #[test]
    fn an_untouched_module_comes_back_the_same() {
        assert_eq!(Module::parse(SMALL).unwrap().finish(), SMALL);
    }

    #[test]
    fn added_functions_are_appended() {
        let mut module = Module::parse(SMALL).unwrap();
        let two = module.declare(&[], &[ValType::I32]);
        module.define(two, &[], &[0x41, 0x02]); // i32.const 2
        module.export("two", two);
        let again = Module::parse(&module.finish()).unwrap();
        assert_eq!(again.exported_function("two"), Some(2));
        assert_eq!(again.functions.count, 2);
        assert_eq!(again.code.count, 2);
        // () -> i32 was already in the module's own types, but only types
        // added here are reused, so one more was added
        assert_eq!(again.types.count, 3);
    }

    #[test]
    fn rejects_what_is_not_a_module() {
        assert!(Module::parse(b"hello").is_err());
        assert!(Module::parse(&SMALL[..20]).is_err());
    }
}
