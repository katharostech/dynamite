//! Types used to define scripting APIs for language adapters

use safer_ffi::prelude::*;
use serde::{Deserialize, Serialize};

use std::{borrow::Cow, collections::HashMap};

pub use ty::Void;
mod ty {
    use safer_ffi::derive_ReprC;

    #[derive_ReprC]
    #[ReprC::opaque]
    /// A type used to represent an untyped pointer
    pub struct Void {
        _private: (),
    }
}

/// A registry of scripted types mapping their unique module path to the type definition.
pub type ScriptApi = HashMap<TypePath, ScriptType>;

/// The path to a scripted type, i.e. the "module" path such as "mygame::physics::RigidBody".
pub type TypePath = String;

/// A script-loaded type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScriptType {
    /// A struct definition
    Struct(StructDefinition),
    /// A function definition
    Function(FunctionDefinition),
}

/// The information necessary to define a component including the component ID and the memory
/// layout.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructDefinition {
    /// The size and alignment of the component
    pub layout: DataLayout,
    /// The type of component this is
    pub component_type: DataType,
    /// The definitions of the methods associated to to the [`method_pointers`] with the same index.
    pub method_definitions: Vec<FunctionDefinition>,
}

impl HasDataLayout for StructDefinition {
    fn get_data_layout(&self) -> DataLayout {
        self.layout
    }
}

/// Implemented by types that can define a [`DataLayout`]
pub trait HasDataLayout {
    fn get_data_layout(&self) -> DataLayout;
}

/// A type memory layout
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DataLayout {
    /// The number of bytes the type takes up
    size: usize,
    /// The alignment of the struct
    align: usize,
}

impl DataLayout {
    pub fn from_size_align(size: usize, align: usize) -> Result<Self, std::alloc::LayoutError> {
        // TODO: Better way to verify the layout parameters?
        std::alloc::Layout::from_size_align(size, align)?;

        Ok(Self { size, align })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn align(&self) -> usize {
        self.align
    }
}

/// A data type, usually of a function argument or return value
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataType {
    /// A pointer to a different type
    Pointer(Box<ScriptType>),
    /// A struct with string field keys
    Struct {
        fields: HashMap<String, StructDefinition>,
    },
    /// A primitive type
    Primitive(Primitive),
}

/// A primitive type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Primitive {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Char,
    Bool,
}

impl HasDataLayout for Primitive {
    #[rustfmt::skip]
    fn get_data_layout(&self) -> DataLayout {
        match self {
            Primitive::Char => DataLayout::from_size_align(1, 1).unwrap(),
            Primitive::Bool => DataLayout::from_size_align(1, 1).unwrap(),
            Primitive::U8   => DataLayout::from_size_align(1, 1).unwrap(),
            Primitive::U16  => DataLayout::from_size_align(2, 2).unwrap(),
            Primitive::U32  => DataLayout::from_size_align(4, 4).unwrap(),
            Primitive::U64  => DataLayout::from_size_align(8, 8).unwrap(),
            Primitive::U128 => DataLayout::from_size_align(16, 16).unwrap(),
            Primitive::I8   => DataLayout::from_size_align(1, 1).unwrap(),
            Primitive::I16  => DataLayout::from_size_align(2, 2).unwrap(),
            Primitive::I32  => DataLayout::from_size_align(4, 4).unwrap(),
            Primitive::I64  => DataLayout::from_size_align(8, 8).unwrap(),
            Primitive::I128 => DataLayout::from_size_align(16, 16).unwrap(),
            Primitive::F32  => DataLayout::from_size_align(4, 4).unwrap(),
            Primitive::F64  => DataLayout::from_size_align(8, 8).unwrap(),
        }
    }
}

/// the definition for a script type's method
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionDefinition {
    /// The arguments of the functions, mapping the arg name to the type path
    pub arguments: HashMap<Cow<'static, str>, TypePath>,
    /// The return value of the function
    pub return_type: Option<TypePath>,
}
