use super::*;

macro_rules! impl_primitive_type {
    ($prim:ident, $variant:ident) => {
        impl HasScriptType for $prim {
            fn script_type() -> ScriptType {
                ScriptType::Primitive(Primitive::$variant)
            }

            fn script_path() -> TypePath {
                stringify!($prim).into()
            }
        }
    };
}

impl_primitive_type!(u8, U8);
impl_primitive_type!(u16, U16);
impl_primitive_type!(u32, U32);
impl_primitive_type!(u64, U64);
impl_primitive_type!(u128, U128);
impl_primitive_type!(i8, I8);
impl_primitive_type!(i16, I16);
impl_primitive_type!(i32, I32);
impl_primitive_type!(i64, I64);
impl_primitive_type!(i128, I128);