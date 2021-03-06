use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "params")]
#[repr(C)]
pub enum Instruction {
    Raw(u8),
    Unreachable,
    Nop,
    Block(u8, Vec<Instruction>),
    Loop(u8, Vec<Instruction>),
    If(u8, Vec<Instruction>, Option<Vec<Instruction>>),
    Br(u32),
    BrIf(u32),
    BrTable(Vec<u32>, u32),
    Return,
    Call(u32),
    CallIndirect(u32),
    Drop,
    Select,
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Load(u32, u32),
    I64Load(u32, u32),
    F32Load(u32, u32),
    F64Load(u32, u32),
    I32Load8S(u32, u32),
    I32Load8U(u32, u32),
    I32Load16S(u32, u32),
    I32Load16U(u32, u32),
    I64Load8S(u32, u32),
    I64Load8U(u32, u32),
    I64Load16S(u32, u32),
    I64Load16U(u32, u32),
    I64Load32S(u32, u32),
    I64Load32U(u32, u32),
    I32Store(u32, u32),
    I64Store(u32, u32),
    F32Store(u32, u32),
    F64Store(u32, u32),
    I32Store8(u32, u32),
    I32Store16(u32, u32),
    I64Store8(u32, u32),
    I64Store16(u32, u32),
    I64Store32(u32, u32),
    MemorySize,
    MemoryGrow,
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,
    I32wrapF64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}
