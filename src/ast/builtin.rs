//! The built-in (standard library) function table.

macro_rules! builtins {
    ($($variant:ident => $name:literal, $min:literal, $max:literal;)*) => {
        /// Built-in (standard library) functions.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Builtin { $($variant,)* }

        impl Builtin {
            pub fn from_name(name: &str) -> Option<Builtin> {
                match name { $($name => Some(Builtin::$variant),)* _ => None }
            }

            pub fn name(self) -> &'static str {
                match self { $(Builtin::$variant => $name,)* }
            }

            /// The names of all built-in functions.
            pub const NAMES: &'static [&'static str] = &[$($name,)*];

            /// Minimum and maximum number of arguments.
            pub fn arity(self) -> (usize, usize) {
                match self { $(Builtin::$variant => ($min, $max),)* }
            }
        }
    };
}

builtins! {
    Sin => "SIN", 1, 1;
    Cos => "COS", 1, 1;
    Tan => "TAN", 1, 1;
    Atn => "ATN", 1, 1;
    Exp => "EXP", 1, 1;
    Abs => "ABS", 1, 1;
    Log => "LOG", 1, 1;
    Sqr => "SQR", 1, 1;
    Int => "INT", 1, 1;
    Fix => "FIX", 1, 1;
    Sgn => "SGN", 1, 1;
    Rnd => "RND", 0, 1;
    Val => "VAL", 1, 1;
    Len => "LEN", 1, 1;
    Asc => "ASC", 1, 1;
    Chr => "CHR$", 1, 1;
    Str => "STR$", 1, 1;
    Left => "LEFT$", 2, 2;
    Right => "RIGHT$", 2, 2;
    Mid => "MID$", 2, 3;
    Instr => "INSTR", 2, 3;
    Space => "SPACE$", 1, 1;
    StringOf => "STRING$", 2, 2;
}
