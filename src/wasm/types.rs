use super::BlockType;
use std::convert::From;

pub trait Type {}

#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    None = 0,
    Any = 1,
    I32 = 2,
    I64 = 3,
    F32 = 4,
    F64 = 5,
    V128 = 6,
    AnyRef = 7,
    AnyFunc = 8,
    NullRef = 9,
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::None
    }
}

impl Type for ValueType {}

impl ValueType {
    pub const LENGTH: usize = 10;

    pub fn get_bytes(&self) -> u32 {
        match self {
            ValueType::I32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::F64 => 8,
            ValueType::V128 => 16,
            ValueType::AnyFunc | ValueType::AnyRef | ValueType::NullRef => 8,
            _ => unreachable!(),
        }
    }
}

impl From<parity_wasm::elements::ValueType> for ValueType {
    fn from(ty: parity_wasm::elements::ValueType) -> Self {
        match ty {
            parity_wasm::elements::ValueType::F32 => ValueType::F32,
            parity_wasm::elements::ValueType::F64 => ValueType::F64,
            parity_wasm::elements::ValueType::I32 => ValueType::I32,
            parity_wasm::elements::ValueType::I64 => ValueType::I64,
            parity_wasm::elements::ValueType::V128 => ValueType::V128,
        }
    }
}

impl From<BlockType> for ValueType {
    fn from(ty: BlockType) -> Self {
        match ty {
            BlockType::Value(v) => ValueType::from(v),
            BlockType::NoResult => ValueType::None,
        }
    }
}

pub struct I32(pub i32);

impl From<i32> for I32 {
    fn from(v: i32) -> I32 {
        I32(v)
    }
}

pub struct I64(pub i64);
impl From<i64> for I64 {
    fn from(v: i64) -> I64 {
        I64(v)
    }
}

pub struct F32(pub f32);
impl From<f32> for F32 {
    fn from(v: f32) -> F32 {
        F32(v)
    }
}

pub struct F64(pub f64);
impl From<f64> for F64 {
    fn from(v: f64) -> F64 {
        F64(v)
    }
}

#[repr(C, align(16))]
pub union V128 {
    i8x16: [i8; 16],
    u8x16: [u8; 16],
    i16x8: [i16; 8],
    u16x8: [u16; 8],
    i32x8: [i32; 4],
    u32x8: [u32; 4],
    i64x2: [i64; 2],
    u64x2: [u64; 2],
}

impl Type for I32 {}
impl Type for I64 {}
impl Type for F32 {}
impl Type for F64 {}
impl Type for V128 {}

impl From<u32> for F32 {
    fn from(v: u32) -> F32 {
        F32(v as f32)
    }
}

impl From<u64> for F64 {
    fn from(v: u64) -> F64 {
        F64(v as f64)
    }
}

impl V128 {
    pub fn zero() -> Self {
        Self { i8x16: [0; 16] }
    }

    pub fn into_u64x2(&self) -> [u64; 2] {
        unsafe { self.u64x2 }
    }
}

impl From<Box<[u8; 16]>> for V128 {
    fn from(v: Box<[u8; 16]>) -> Self {
        Self { u8x16: *v }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalType {
    mutable: bool,
    ty: ValueType,
}

impl Type for GlobalType {}

impl From<parity_wasm::elements::GlobalType> for GlobalType {
    fn from(v: parity_wasm::elements::GlobalType) -> GlobalType {
        GlobalType {
            mutable: v.is_mutable(),
            ty: ValueType::from(v.content_type()),
        }
    }
}

impl GlobalType {
    pub fn value_type(&self) -> &ValueType {
        &self.ty
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }
}

#[derive(Clone, Default, Debug)]
pub struct FunctionType {
    res: Option<ValueType>,
    params: Vec<ValueType>,
}

impl Type for FunctionType {}

impl From<parity_wasm::elements::FunctionType> for FunctionType {
    fn from(func_type: parity_wasm::elements::FunctionType) -> Self {
        let res = if let Some(res_type) = func_type.return_type() {
            Some(ValueType::from(res_type))
        } else {
            None
        };
        let params = func_type
            .params()
            .iter()
            .map(|t| ValueType::from(*t))
            .collect();

        Self { res, params }
    }
}

impl FunctionType {
    pub fn new(params: Vec<ValueType>, res: Option<ValueType>) -> Self {
        Self { res, params }
    }
    pub fn res(&self) -> Option<ValueType> {
        self.res
    }

    pub fn params(&self) -> &[ValueType] {
        &self.params
    }
}

#[derive(Clone, Copy)]
pub struct MemoryType {
    min: u32,
    max: Option<u32>,
    shared: bool
}

impl Type for MemoryType {}

impl From<parity_wasm::elements::MemoryType> for MemoryType {
    fn from(mem_type: parity_wasm::elements::MemoryType) -> Self {
        let min = mem_type.limits().initial();
        let max = mem_type.limits().maximum();
        let shared = mem_type.limits().shared();
        Self { min, max, shared }
    }
}

impl MemoryType {
    pub fn min_pages(&self) -> u32 { self.min }

    pub fn max_pages(&self) -> Option<u32> { self.max }

    pub fn is_shared(&self) -> bool { self.shared }
}
