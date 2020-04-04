#![no_std]
#[macro_use]
extern crate alloc;
extern crate serde;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::convert::TryInto;
use serde::{Deserialize, Serialize};
use webassembly::*;

pub trait WasmValueTypes {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, &'static str>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(C)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

impl TryFrom<u8> for ValueType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            I32 => Ok(ValueType::I32),
            I64 => Ok(ValueType::I64),
            F32 => Ok(ValueType::F32),
            F64 => Ok(ValueType::F64),
            _ => Err("could not convert data type"),
        }
    }
}

impl WasmValueTypes for Vec<u8> {
    fn try_to_value_types(self) -> Result<Vec<ValueType>, &'static str> {
        let mut r = vec![];
        for v in self {
            r.push(v.try_into()?);
        }
        Ok(r)
    }
}

impl TryFrom<&u8> for ValueType {
    type Error = &'static str;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            I32 => Ok(ValueType::I32),
            I64 => Ok(ValueType::I64),
            F32 => Ok(ValueType::F32),
            F64 => Ok(ValueType::F64),
            _ => Err("could not convert data type"),
        }
    }
}

fn tag(tag: &[u8]) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), &'static str> + '_ {
    move |input: &[u8]| {
        if tag.len() > input.len() {
            return Err("trying to tag too many bytes");
        }
        for i in 0..tag.len() {
            if tag[i] != input[i] {
                return Err("did not match tag");
            }
        }
        Ok((&input[tag.len()..], &input[..tag.len()]))
    }
}

fn take(num: usize) -> impl Fn(&[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    move |input: &[u8]| {
        if num > input.len() {
            return Err("trying to take too many bytes");
        }
        Ok((&input[num..], &input[..num]))
    }
}

fn many_n<'a, T>(
    n: usize,
    f: impl Fn(&'a [u8]) -> Result<(&'a [u8], T), &'static str>,
) -> impl Fn(&'a [u8]) -> Result<(&'a [u8], Vec<T>), &'static str> {
    move |input: &[u8]| {
        let mut v = vec![];
        let mut ip = input;
        loop {
            if n == v.len() {
                break;
            }
            match f(ip) {
                Ok((input, item)) => {
                    v.push(item);
                    ip = input;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok((ip, v))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionType {
    pub inputs: Vec<ValueType>,
    pub outputs: Vec<ValueType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "value_type", content = "content")]
#[repr(C)]
pub enum WasmType {
    #[serde(rename(serialize = "function"))]
    Function(FunctionType),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TypeSection {
    pub types: Vec<WasmType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionSection {
    pub function_types: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CodeBlock {
    pub locals: Vec<(u32, ValueType)>,
    pub code_expression: Vec<Instruction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CodeSection {
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportView<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "export_type", content = "content")]
#[repr(C)]
pub enum WasmExportView<'a> {
    #[serde(rename(serialize = "function"))]
    #[serde(borrow)]
    Function(ExportView<'a>),
    #[serde(rename(serialize = "table"))]
    #[serde(borrow)]
    Table(ExportView<'a>),
    #[serde(rename(serialize = "memory"))]
    #[serde(borrow)]
    Memory(ExportView<'a>),
    #[serde(rename(serialize = "global"))]
    #[serde(borrow)]
    Global(ExportView<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportSectionView<'a> {
    #[serde(borrow)]
    pub exports: Vec<WasmExportView<'a>>,
}

impl<'a> ExportSectionView<'a> {
    fn to_owned(&self) -> ExportSection {
        ExportSection {
            exports: self
                .exports
                .iter()
                .map(|x| match x {
                    WasmExportView::Function(x) => WasmExport::Function(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Global(x) => WasmExport::Global(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Memory(x) => WasmExport::Memory(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                    WasmExportView::Table(x) => WasmExport::Table(Export {
                        name: x.name.to_string(),
                        index: x.index,
                    }),
                })
                .collect::<Vec<WasmExport>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Export {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "export_type", content = "content")]
#[repr(C)]
pub enum WasmExport {
    #[serde(rename(serialize = "function"))]
    Function(Export),
    #[serde(rename(serialize = "table"))]
    Table(Export),
    #[serde(rename(serialize = "memory"))]
    Memory(Export),
    #[serde(rename(serialize = "global"))]
    Global(Export),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ExportSection {
    pub exports: Vec<WasmExport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub type_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub value_type: ValueType,
    pub is_mutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemoryImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableImportView<'a> {
    #[serde(borrow)]
    pub module_name: &'a str,
    #[serde(borrow)]
    pub name: &'a str,
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "import_type", content = "content")]
#[repr(C)]
pub enum WasmImportView<'a> {
    #[serde(rename(serialize = "function"))]
    #[serde(borrow)]
    Function(FunctionImportView<'a>),
    #[serde(rename(serialize = "global"))]
    #[serde(borrow)]
    Global(GlobalImportView<'a>),
    #[serde(rename(serialize = "memory"))]
    #[serde(borrow)]
    Memory(MemoryImportView<'a>),
    #[serde(rename(serialize = "table"))]
    #[serde(borrow)]
    Table(TableImportView<'a>),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct FunctionImport {
    pub module_name: String,
    pub name: String,
    pub type_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalImport {
    pub module_name: String,
    pub name: String,
    pub value_type: ValueType,
    pub is_mutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemoryImport {
    pub module_name: String,
    pub name: String,
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableImport {
    pub module_name: String,
    pub name: String,
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "import_type", content = "content")]
#[repr(C)]
pub enum WasmImport {
    #[serde(rename(serialize = "function"))]
    Function(FunctionImport),
    #[serde(rename(serialize = "global"))]
    Global(GlobalImport),
    #[serde(rename(serialize = "memory"))]
    Memory(MemoryImport),
    #[serde(rename(serialize = "table"))]
    Table(TableImport),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ImportSection {
    pub imports: Vec<WasmImport>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ImportSectionView<'a> {
    #[serde(borrow)]
    pub imports: Vec<WasmImportView<'a>>,
}

impl<'a> ImportSectionView<'a> {
    fn to_owned(&self) -> ImportSection {
        ImportSection {
            imports: self
                .imports
                .iter()
                .map(|x| match x {
                    WasmImportView::Function(x) => WasmImport::Function(FunctionImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        type_index: x.type_index,
                    }),
                    WasmImportView::Global(x) => WasmImport::Global(GlobalImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        value_type: x.value_type,
                        is_mutable: x.is_mutable,
                    }),
                    WasmImportView::Memory(x) => WasmImport::Memory(MemoryImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        min_pages: x.min_pages,
                        max_pages: x.max_pages,
                    }),
                    WasmImportView::Table(x) => WasmImport::Table(TableImport {
                        module_name: x.module_name.to_string(),
                        name: x.name.to_string(),
                        element_type: x.element_type,
                        min: x.min,
                        max: x.max,
                    }),
                })
                .collect::<Vec<WasmImport>>(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct WasmMemory {
    pub min_pages: usize,
    pub max_pages: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct MemorySection {
    pub memories: Vec<WasmMemory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct StartSection {
    pub start_function: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Global {
    pub value_type: ValueType,
    pub is_mutable: bool,
    pub value_expression: Vec<Instruction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct GlobalSection {
    pub globals: Vec<Global>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Table {
    pub element_type: u8,
    pub min: usize,
    pub max: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct TableSection {
    pub tables: Vec<Table>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataBlockView<'a> {
    pub memory: usize,
    pub offset_expression: Vec<Instruction>,
    #[serde(borrow)]
    pub data: &'a [u8],
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataSectionView<'a> {
    #[serde(borrow)]
    pub data_blocks: Vec<DataBlockView<'a>>,
}

impl<'a> DataSectionView<'a> {
    fn to_owned(&self) -> DataSection {
        DataSection {
            data_blocks: self
                .data_blocks
                .iter()
                .map(|x| DataBlock {
                    memory: x.memory,
                    offset_expression: x.offset_expression.clone(),
                    data: x.data.to_vec(),
                })
                .collect::<Vec<DataBlock>>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataBlock {
    pub memory: usize,
    pub offset_expression: Vec<Instruction>,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct DataSection {
    pub data_blocks: Vec<DataBlock>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CustomSectionView<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    #[serde(borrow)]
    pub data: &'a [u8],
}

impl<'a> CustomSectionView<'a> {
    fn to_owned(&self) -> CustomSection {
        CustomSection {
            name: self.name.to_string(),
            data: self.data.to_vec(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct CustomSection {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct WasmElement {
    pub table: usize,
    pub value_expression: Vec<Instruction>,
    pub functions: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ElementSection {
    pub elements: Vec<WasmElement>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "section_type", content = "content")]
#[repr(C)]
pub enum SectionView<'a> {
    #[serde(rename(serialize = "type"))]
    Type(TypeSection),
    #[serde(rename(serialize = "function"))]
    Function(FunctionSection),
    #[serde(rename(serialize = "code"))]
    Code(CodeSection),
    #[serde(rename(serialize = "export"))]
    #[serde(borrow)]
    Export(ExportSectionView<'a>),
    #[serde(rename(serialize = "import"))]
    #[serde(borrow)]
    Import(ImportSectionView<'a>),
    #[serde(rename(serialize = "memory"))]
    Memory(MemorySection),
    #[serde(rename(serialize = "start"))]
    Start(StartSection),
    #[serde(rename(serialize = "global"))]
    Global(GlobalSection),
    #[serde(rename(serialize = "table"))]
    Table(TableSection),
    #[serde(rename(serialize = "data"))]
    #[serde(borrow)]
    Data(DataSectionView<'a>),
    #[serde(rename(serialize = "custom"))]
    #[serde(borrow)]
    Custom(CustomSectionView<'a>),
    #[serde(rename(serialize = "element"))]
    Element(ElementSection),
}

impl<'a> SectionView<'a> {
    fn to_owned(&self) -> Section {
        match self {
            SectionView::Type(s) => Section::Type(s.clone()),
            SectionView::Function(s) => Section::Function(s.clone()),
            SectionView::Code(s) => Section::Code(s.clone()),
            SectionView::Export(s) => Section::Export(s.to_owned()),
            SectionView::Import(s) => Section::Import(s.to_owned()),
            SectionView::Memory(s) => Section::Memory(s.clone()),
            SectionView::Start(s) => Section::Start(s.clone()),
            SectionView::Global(s) => Section::Global(s.clone()),
            SectionView::Table(s) => Section::Table(s.clone()),
            SectionView::Data(s) => Section::Data(s.to_owned()),
            SectionView::Custom(s) => Section::Custom(s.to_owned()),
            SectionView::Element(s) => Section::Element(s.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "section_type", content = "content")]
#[repr(C)]
pub enum Section {
    #[serde(rename(serialize = "type"))]
    Type(TypeSection),
    #[serde(rename(serialize = "function"))]
    Function(FunctionSection),
    #[serde(rename(serialize = "code"))]
    Code(CodeSection),
    #[serde(rename(serialize = "export"))]
    Export(ExportSection),
    #[serde(rename(serialize = "import"))]
    Import(ImportSection),
    #[serde(rename(serialize = "memory"))]
    Memory(MemorySection),
    #[serde(rename(serialize = "start"))]
    Start(StartSection),
    #[serde(rename(serialize = "global"))]
    Global(GlobalSection),
    #[serde(rename(serialize = "table"))]
    Table(TableSection),
    #[serde(rename(serialize = "data"))]
    Data(DataSection),
    #[serde(rename(serialize = "custom"))]
    Custom(CustomSection),
    #[serde(rename(serialize = "element"))]
    Element(ElementSection),
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct ProgramView<'a> {
    #[serde(borrow)]
    pub sections: Vec<SectionView<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Program {
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "params")]
#[repr(C)]
pub enum Instruction {
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
    F32AbS,
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
    F64AbS,
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

fn wasm_u32(input: &[u8]) -> Result<(&[u8], u32), &'static str> {
    let (i, byte_count) = match input.try_extract_u32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i))
}

fn wasm_i32(input: &[u8]) -> Result<(&[u8], i32, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_i64(input: &[u8]) -> Result<(&[u8], i64, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_i64(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f32(input: &[u8]) -> Result<(&[u8], f32, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f32(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_f64(input: &[u8]) -> Result<(&[u8], f64, &[u8]), &'static str> {
    let original_input = input;
    let (i, byte_count) = match input.try_extract_f64(0) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let (input, _) = take(byte_count as usize)(input)?;
    Ok((input, i, &original_input[..byte_count]))
}

fn wasm_string(input: &[u8]) -> Result<(&[u8], &str), &'static str> {
    let (input, num_chars) = wasm_u32(input)?;
    let (input, chars) = take(num_chars as usize)(input)?;
    let s = match alloc::str::from_utf8(chars) {
        Ok(b) => b,
        Err(_) => return Err("could not parse utf8 string"),
    };
    Ok((input, s))
}

fn wasm_global_type(input: &[u8]) -> Result<(&[u8], ValueType, bool), &'static str> {
    let (input, global_value_type) = take(1)(input)?;
    let (input, global_type) = take(1)(input)?;
    Ok((
        input,
        global_value_type[0].try_into()?,
        global_type[0] == MUTABLE,
    ))
}

fn wasm_limit(input: &[u8]) -> Result<(&[u8], usize, Option<usize>), &'static str> {
    let (input, mem_type) = take(1)(input)?;
    match mem_type[0] {
        LIMIT_MIN_MAX => {
            let (input, min) = wasm_u32(input)?;
            let (input, max) = wasm_u32(input)?;
            Ok((input, min as usize, Some(max as usize)))
        }
        LIMIT_MIN => {
            let (input, min) = wasm_u32(input)?;
            Ok((input, min as usize, None))
        }
        _ => Err("unhandled memory type"),
    }
}

fn wasm_instruction(op: u8, input: &[u8]) -> Result<(&[u8], Instruction), &'static str> {
    let mut ip = input;
    let instruction;

    match op {
        UNREACHABLE => instruction = Instruction::Unreachable,
        NOP => instruction = Instruction::Nop,
        RETURN => instruction = Instruction::Return,

        BLOCK => {
            let (input, block_type) = take(1)(input)?;
            let (input, block_instructions) = wasm_expression(input)?;
            instruction = Instruction::Block(block_type[0], block_instructions);
            ip = input;
        }

        LOOP => {
            let (input, block_type) = take(1)(input)?;
            let (input, loop_instructions) = wasm_expression(input)?;
            instruction = Instruction::Loop(block_type[0], loop_instructions);
            ip = input;
        }

        IF => {
            let (input, block_type) = take(1)(input)?;
            let (input, if_instructions, else_instructions) = wasm_if_else(input)?;
            instruction = Instruction::If(block_type[0], if_instructions, else_instructions);
            ip = input;
        }

        BR => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::Br(idx);
            ip = input;
        }

        BR_IF => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::BrIf(idx);
            ip = input;
        }

        BR_TABLE => {
            let (input, num_labels) = wasm_u32(input)?;
            let parse_label = many_n(num_labels as usize, |input| wasm_u32(input));
            let (input, labels) = parse_label(input)?;
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::BrTable(labels, idx);
            ip = input;
        }

        CALL => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::Call(idx);
            ip = input;
        }

        CALL_INDIRECT => {
            let (input, idx) = wasm_u32(input)?;
            let (input, _) = wasm_u32(input)?;
            instruction = Instruction::Call(idx);
            ip = input;
        }

        DROP => instruction = Instruction::Drop,
        SELECT => instruction = Instruction::Select,
        I32_CONST => {
            let (input, c, _) = wasm_i32(input)?;
            instruction = Instruction::I32Const(c);
            ip = input;
        }
        I64_CONST => {
            let (input, c, _) = wasm_i64(input)?;
            instruction = Instruction::I64Const(c);
            ip = input;
        }

        F32_CONST => {
            let (input, c, _) = wasm_f32(input)?;
            instruction = Instruction::F32Const(c);
            ip = input;
        }

        F64_CONST => {
            let (input, c, _) = wasm_f64(input)?;
            instruction = Instruction::F64Const(c);
            ip = input;
        }
        LOCAL_GET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalGet(idx);
            ip = input;
        }
        LOCAL_SET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalSet(idx);
            ip = input;
        }
        LOCAL_TEE => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::LocalTee(idx);
            ip = input;
        }
        GLOBAL_GET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::GlobalGet(idx);
            ip = input;
        }
        GLOBAL_SET => {
            let (input, idx) = wasm_u32(input)?;
            instruction = Instruction::GlobalSet(idx);
            ip = input;
        }
        I32_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load(align, offset);
            ip = input;
        }

        I64_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load(align, offset);
            ip = input;
        }

        F32_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F32Load(align, offset);
            ip = input;
        }

        F64_LOAD => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F64Load(align, offset);
            ip = input;
        }

        I32_LOAD8_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load8S(align, offset);
            ip = input;
        }

        I32_LOAD8_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load8U(align, offset);
            ip = input;
        }

        I32_LOAD16_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load16S(align, offset);
            ip = input;
        }

        I32_LOAD16_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Load16U(align, offset);
            ip = input;
        }

        I64_LOAD8_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load8S(align, offset);
            ip = input;
        }

        I64_LOAD8_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load8U(align, offset);
            ip = input;
        }

        I64_LOAD16_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load16S(align, offset);
            ip = input;
        }

        I64_LOAD16_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load16U(align, offset);
            ip = input;
        }

        I64_LOAD32_S => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load32S(align, offset);
            ip = input;
        }

        I64_LOAD32_U => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Load32U(align, offset);
            ip = input;
        }

        I32_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store(align, offset);
            ip = input;
        }

        I64_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store(align, offset);
            ip = input;
        }

        F32_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F32Store(align, offset);
            ip = input;
        }
        F64_STORE => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::F64Store(align, offset);
            ip = input;
        }

        I32_STORE8 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store8(align, offset);
            ip = input;
        }

        I32_STORE16 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I32Store16(align, offset);
            ip = input;
        }

        I64_STORE8 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store8(align, offset);
            ip = input;
        }

        I64_STORE16 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store16(align, offset);
            ip = input;
        }

        I64_STORE32 => {
            let (input, align) = wasm_u32(input)?;
            let (input, offset) = wasm_u32(input)?;
            instruction = Instruction::I64Store32(align, offset);
            ip = input;
        }

        MEMORY_GROW => {
            let (input, _) = wasm_u32(input)?;
            instruction = Instruction::MemoryGrow;
            ip = input;
        }

        MEMORY_SIZE => {
            let (input, _) = wasm_u32(input)?;
            instruction = Instruction::MemorySize;
            ip = input;
        }

        I32_EQZ => instruction = Instruction::I32Eqz,
        I32_EQ => instruction = Instruction::I32Eq,
        I32_NE => instruction = Instruction::I32Ne,
        I32_LT_S => instruction = Instruction::I32LtS,
        I32_LT_U => instruction = Instruction::I32LtU,
        I32_GT_S => instruction = Instruction::I32GtS,
        I32_GT_U => instruction = Instruction::I32GtU,
        I32_LE_S => instruction = Instruction::I32LeS,
        I32_LE_U => instruction = Instruction::I32LeU,
        I32_GE_S => instruction = Instruction::I32GeS,
        I32_GE_U => instruction = Instruction::I32GeU,
        I64_EQZ => instruction = Instruction::I64Eqz,
        I64_EQ => instruction = Instruction::I64Eq,
        I64_NE => instruction = Instruction::I64Ne,
        I64_LT_S => instruction = Instruction::I64LtS,
        I64_LT_U => instruction = Instruction::I64LtU,
        I64_GT_S => instruction = Instruction::I64GtS,
        I64_GT_U => instruction = Instruction::I64GtU,
        I64_LE_S => instruction = Instruction::I64LeS,
        I64_LE_U => instruction = Instruction::I64LeU,
        I64_GE_S => instruction = Instruction::I64GeS,
        I64_GE_U => instruction = Instruction::I64GeU,
        F32_EQ => instruction = Instruction::F32Eq,
        F32_NE => instruction = Instruction::F32Ne,
        F32_LT => instruction = Instruction::F32Lt,
        F32_GT => instruction = Instruction::F32Gt,
        F32_LE => instruction = Instruction::F32Le,
        F32_GE => instruction = Instruction::F32Ge,
        F64_EQ => instruction = Instruction::F64Eq,
        F64_NE => instruction = Instruction::F64Ne,
        F64_LT => instruction = Instruction::F64Lt,
        F64_GT => instruction = Instruction::F64Gt,
        F64_LE => instruction = Instruction::F64Le,
        F64_GE => instruction = Instruction::F64Ge,
        I32_CLZ => instruction = Instruction::I32Clz,
        I32_CTZ => instruction = Instruction::I32Ctz,
        I32_POPCNT => instruction = Instruction::I32Popcnt,
        I32_ADD => instruction = Instruction::I32Add,
        I32_SUB => instruction = Instruction::I32Sub,
        I32_MUL => instruction = Instruction::I32Mul,
        I32_DIV_S => instruction = Instruction::I32DivS,
        I32_DIV_U => instruction = Instruction::I32DivU,
        I32_REM_S => instruction = Instruction::I32RemS,
        I32_REM_U => instruction = Instruction::I32RemU,
        I32_AND => instruction = Instruction::I32And,
        I32_OR => instruction = Instruction::I32Or,
        I32_XOR => instruction = Instruction::I32Xor,
        I32_SHL => instruction = Instruction::I32Shl,
        I32_SHR_S => instruction = Instruction::I32ShrS,
        I32_SHR_U => instruction = Instruction::I32ShrU,
        I32_ROTL => instruction = Instruction::I32Rotl,
        I32_ROTR => instruction = Instruction::I32Rotr,
        I64_CLZ => instruction = Instruction::I64Clz,
        I64_CTZ => instruction = Instruction::I64Ctz,
        I64_POPCNT => instruction = Instruction::I64Popcnt,
        I64_ADD => instruction = Instruction::I64Add,
        I64_SUB => instruction = Instruction::I64Sub,
        I64_MUL => instruction = Instruction::I64Mul,
        I64_DIV_S => instruction = Instruction::I64DivS,
        I64_DIV_U => instruction = Instruction::I64DivU,
        I64_REM_S => instruction = Instruction::I64RemS,
        I64_REM_U => instruction = Instruction::I64RemU,
        I64_AND => instruction = Instruction::I64And,
        I64_OR => instruction = Instruction::I64Or,
        I64_XOR => instruction = Instruction::I64Xor,
        I64_SHL => instruction = Instruction::I64Shl,
        I64_SHR_S => instruction = Instruction::I64ShrS,
        I64_SHR_U => instruction = Instruction::I64ShrU,
        I64_ROTL => instruction = Instruction::I64Rotl,
        I64_ROTR => instruction = Instruction::I64Rotr,
        F32_ABS => instruction = Instruction::F32AbS,
        F32_NEG => instruction = Instruction::F32Neg,
        F32_CEIL => instruction = Instruction::F32Ceil,
        F32_FLOOR => instruction = Instruction::F32Floor,
        F32_TRUNC => instruction = Instruction::F32Trunc,
        F32_NEAREST => instruction = Instruction::F32Nearest,
        F32_SQRT => instruction = Instruction::F32Sqrt,
        F32_ADD => instruction = Instruction::F32Add,
        F32_SUB => instruction = Instruction::F32Sub,
        F32_MUL => instruction = Instruction::F32Mul,
        F32_DIV => instruction = Instruction::F32Div,
        F32_MIN => instruction = Instruction::F32Min,
        F32_MAX => instruction = Instruction::F32Max,
        F32_COPYSIGN => instruction = Instruction::F32Copysign,
        F64_ABS => instruction = Instruction::F64AbS,
        F64_NEG => instruction = Instruction::F64Neg,
        F64_CEIL => instruction = Instruction::F64Ceil,
        F64_FLOOR => instruction = Instruction::F64Floor,
        F64_TRUNC => instruction = Instruction::F64Trunc,
        F64_NEAREST => instruction = Instruction::F64Nearest,
        F64_SQRT => instruction = Instruction::F64Sqrt,
        F64_ADD => instruction = Instruction::F64Add,
        F64_SUB => instruction = Instruction::F64Sub,
        F64_MUL => instruction = Instruction::F64Mul,
        F64_DIV => instruction = Instruction::F64Div,
        F64_MIN => instruction = Instruction::F64Min,
        F64_MAX => instruction = Instruction::F64Max,
        F64_COPYSIGN => instruction = Instruction::F64Copysign,
        I32_WRAP_F64 => instruction = Instruction::I32wrapF64,
        I32_TRUNC_S_F32 => instruction = Instruction::I32TruncSF32,
        I32_TRUNC_U_F32 => instruction = Instruction::I32TruncUF32,
        I32_TRUNC_S_F64 => instruction = Instruction::I32TruncSF64,
        I32_TRUNC_U_F64 => instruction = Instruction::I32TruncUF64,
        I64_EXTEND_S_I32 => instruction = Instruction::I64ExtendSI32,
        I64_EXTEND_U_I32 => instruction = Instruction::I64ExtendUI32,
        I64_TRUNC_S_F32 => instruction = Instruction::I64TruncSF32,
        I64_TRUNC_U_F32 => instruction = Instruction::I64TruncUF32,
        I64_TRUNC_S_F64 => instruction = Instruction::I64TruncSF64,
        I64_TRUNC_U_F64 => instruction = Instruction::I64TruncUF64,
        F32_CONVERT_S_I32 => instruction = Instruction::F32ConvertSI32,
        F32_CONVERT_U_I32 => instruction = Instruction::F32ConvertUI32,
        F32_CONVERT_S_I64 => instruction = Instruction::F32ConvertSI64,
        F32_CONVERT_U_I64 => instruction = Instruction::F32ConvertUI64,
        F32_DEMOTE_F64 => instruction = Instruction::F32DemoteF64,
        F64_CONVERT_S_I32 => instruction = Instruction::F64ConvertSI32,
        F64_CONVERT_U_I32 => instruction = Instruction::F64ConvertUI32,
        F64_CONVERT_S_I64 => instruction = Instruction::F64ConvertSI64,
        F64_CONVERT_U_I64 => instruction = Instruction::F64ConvertUI64,
        F64_PROMOTE_F32 => instruction = Instruction::F64PromoteF32,
        I32_REINTERPRET_F32 => instruction = Instruction::I32ReinterpretF32,
        I64_REINTERPRET_F64 => instruction = Instruction::I64ReinterpretF64,
        F32_REINTERPRET_I32 => instruction = Instruction::F32ReinterpretI32,
        F64_REINTERPRET_I64 => instruction = Instruction::F64ReinterpretI64,
        _ => return Err("unknown expression"),
    };
    Ok((ip, instruction))
}

fn wasm_expression(input: &[u8]) -> Result<(&[u8], Vec<Instruction>), &'static str> {
    let mut instructions = vec![];
    let mut ip = input;
    loop {
        let (input, op) = take(1)(ip)?;
        ip = input;
        match op[0] {
            END => {
                ip = input;
                break;
            }

            _ => {
                let (input, instruction) = wasm_instruction(op[0], ip)?;
                instructions.push(instruction);
                ip = input;
            }
        }
    }
    Ok((ip, instructions))
}

type Instructions = Vec<Instruction>;

fn wasm_if_else(input: &[u8]) -> Result<(&[u8], Instructions, Option<Instructions>), &'static str> {
    let mut if_instructions = vec![];
    let mut else_instructions = vec![];
    let mut ip = input;
    let mut more = false;
    loop {
        let (input, op) = take(1)(ip)?;
        ip = input;
        match op[0] {
            END => {
                break;
            }
            ELSE => {
                more = true;
                break;
            }

            _ => {
                let (input, instruction) = wasm_instruction(op[0], ip)?;
                if_instructions.push(instruction);
                ip = input;
            }
        }
    }
    if more {
        loop {
            let (input, op) = take(1)(ip)?;
            ip = input;
            match op[0] {
                END => {
                    break;
                }

                _ => {
                    let (input, instruction) = wasm_instruction(op[0], ip)?;
                    else_instructions.push(instruction);
                    ip = input;
                }
            }
        }
        Ok((ip, if_instructions, Some(else_instructions)))
    } else {
        Ok((ip, if_instructions, None))
    }
}

fn section(input: &[u8]) -> Result<(&[u8], SectionView), &'static str> {
    let (input, id) = take(1)(input)?;
    let (input, section_length) = wasm_u32(input)?;

    match id[0] {
        SECTION_TYPE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, wasm_type) = take(1)(input)?;
                match wasm_type[0] {
                    FUNC => {
                        let (input, num_inputs) = wasm_u32(input)?;
                        let (input, inputs) = take(num_inputs as usize)(input)?;
                        let (input, num_outputs) = wasm_u32(input)?;
                        let (input, outputs) = take(num_outputs as usize)(input)?;
                        Ok((
                            input,
                            WasmType::Function(FunctionType {
                                inputs: inputs.to_vec().try_to_value_types()?,
                                outputs: outputs.to_vec().try_to_value_types()?,
                            }),
                        ))
                    }
                    _ => Err("unknown type"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Type(TypeSection { types: items })))
        }
        SECTION_FUNCTION => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| wasm_u32(input));
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Function(FunctionSection {
                    function_types: items,
                }),
            ))
        }
        SECTION_START => {
            let (input, start_function) = wasm_u32(input)?;
            Ok((
                input,
                SectionView::Start(StartSection {
                    start_function: start_function as usize,
                }),
            ))
        }
        SECTION_EXPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, name) = wasm_string(input)?;
                let (input, export_type) = take(1)(input)?;
                let (input, export_index) = wasm_u32(input)?;
                match export_type[0] {
                    DESC_FUNCTION => Ok((
                        input,
                        WasmExportView::Function(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_MEMORY => Ok((
                        input,
                        WasmExportView::Memory(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_GLOBAL => Ok((
                        input,
                        WasmExportView::Global(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    DESC_TABLE => Ok((
                        input,
                        WasmExportView::Table(ExportView {
                            name,
                            index: export_index as usize,
                        }),
                    )),
                    _ => Err("unknown export"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Export(ExportSectionView { exports: items }),
            ))
        }
        SECTION_CODE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, _) = wasm_u32(input)?;
                let (input, num_local_vecs) = wasm_u32(input)?;
                let parse_local_vecs = many_n(num_local_vecs as usize, |input| {
                    let (input, num_locals) = wasm_u32(input)?;
                    let (input, local_type) = take(1 as usize)(input)?;
                    Ok((input, (num_locals, local_type[0].try_into()?)))
                });
                let (input, local_vectors) = parse_local_vecs(input)?;

                let (input, code_expression) = wasm_expression(input)?;
                Ok((
                    input,
                    CodeBlock {
                        locals: local_vectors,
                        code_expression,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Code(CodeSection { code_blocks: items })))
        }
        SECTION_IMPORT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, module_name) = wasm_string(input)?;
                let (input, name) = wasm_string(input)?;
                let (input, import_type) = take(1)(input)?;
                match import_type[0] {
                    DESC_FUNCTION => {
                        let (input, type_index) = wasm_u32(input)?;
                        Ok((
                            input,
                            WasmImportView::Function(FunctionImportView {
                                module_name,
                                name,
                                type_index: type_index as usize,
                            }),
                        ))
                    }
                    DESC_GLOBAL => {
                        let (input, min_pages, max_pages) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImportView::Memory(MemoryImportView {
                                module_name,
                                name,
                                min_pages,
                                max_pages,
                            }),
                        ))
                    }
                    DESC_TABLE => {
                        let (input, element_type) = take(1)(input)?;
                        let (input, min, max) = wasm_limit(input)?;
                        Ok((
                            input,
                            WasmImportView::Table(TableImportView {
                                module_name,
                                name,
                                element_type: element_type[0],
                                min,
                                max,
                            }),
                        ))
                    }
                    DESC_MEMORY => {
                        let (input, value_type, is_mutable) = wasm_global_type(input)?;
                        Ok((
                            input,
                            WasmImportView::Global(GlobalImportView {
                                module_name,
                                name,
                                value_type,
                                is_mutable,
                            }),
                        ))
                    }
                    _ => Err("unknown export"),
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Import(ImportSectionView { imports: items }),
            ))
        }
        SECTION_GLOBAL => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, value_type, is_mutable) = wasm_global_type(input)?;
                let (input, expression) = wasm_expression(input)?;
                Ok((
                    input,
                    Global {
                        value_type,
                        is_mutable,
                        value_expression: expression,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Global(GlobalSection { globals: items })))
        }
        SECTION_CUSTOM => {
            let mut name_bytes_length = 0;
            let (num_chars, byte_count) = match input.try_extract_u32(0) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };
            let (input, _) = take(byte_count as usize)(input)?;
            let (input, chars) = take(num_chars as usize)(input)?;
            name_bytes_length += byte_count + num_chars as usize;
            let name = match core::str::from_utf8(chars) {
                Ok(b) => b,
                Err(_) => return Err("could not parse utf8 string"),
            };
            let (input, bytes) =
                take((section_length as usize - name_bytes_length) as usize)(input)?;
            Ok((
                input,
                SectionView::Custom(CustomSectionView { name, data: bytes }),
            ))
        }
        SECTION_TABLE => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, element_type) = take(1)(input)?;
                if element_type[0] == ANYFUNC {
                    let (input, min, max) = wasm_limit(input)?;
                    Ok((
                        input,
                        Table {
                            element_type: element_type[0],
                            min,
                            max,
                        },
                    ))
                } else {
                    Err("unknown table type")
                }
            });
            let (input, items) = parse_items(input)?;
            Ok((input, SectionView::Table(TableSection { tables: items })))
        }
        SECTION_DATA => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, mem_index) = wasm_u32(input)?;
                let (input, offset_expression) = wasm_expression(input)?;
                let (input, data_len) = wasm_u32(input)?;
                let (input, data) = take(data_len as usize)(input)?;
                Ok((
                    input,
                    DataBlockView {
                        memory: mem_index as usize,
                        offset_expression,
                        data,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Data(DataSectionView { data_blocks: items }),
            ))
        }
        SECTION_MEMORY => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, min, max) = wasm_limit(input)?;
                Ok((
                    input,
                    WasmMemory {
                        min_pages: min,
                        max_pages: max,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Memory(MemorySection { memories: items }),
            ))
        }
        SECTION_ELEMENT => {
            let (input, num_items) = wasm_u32(input)?;
            let parse_items = many_n(num_items as usize, |input| {
                let (input, table) = wasm_u32(input)?;
                let (input, expression) = wasm_expression(input)?;
                let (input, num_functions) = wasm_u32(input)?;
                let parse_functions = many_n(num_functions as usize, |input| {
                    let (input, i) = wasm_u32(input)?;
                    Ok((input, i as usize))
                });
                let (input, functions) = parse_functions(input)?;
                Ok((
                    input,
                    WasmElement {
                        table: table as usize,
                        value_expression: expression,
                        functions,
                    },
                ))
            });
            let (input, items) = parse_items(input)?;
            Ok((
                input,
                SectionView::Element(ElementSection { elements: items }),
            ))
        }
        _ => Err("unknow section"),
    }
}

fn wasm_module(input: &[u8]) -> Result<ProgramView, &'static str> {
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _) = tag(VERSION_1)(input)?;
    let mut sections = vec![];
    let mut ip = input;
    let mut p = ProgramView { sections: vec![] };
    loop {
        match section(ip) {
            Ok((input, item)) => {
                sections.push(item);
                ip = input;
            }
            Err(e) => {
                if ip.is_empty() {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
    }
    p.sections = sections;
    Ok(p)
}

impl<'p> ProgramView<'p> {
    pub fn find_exported_function<'a>(
        &'a self,
        name: &str,
    ) -> Result<&'a ExportView, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let SectionView::Export(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(SectionView::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| {
                if let SectionView::Code(_) = x {
                    true
                } else {
                    false
                }
            });
            if let Some(SectionView::Code(_)) = result {
                let result = export_section.exports.iter().find(|x| {
                    if let WasmExportView::Function(f) = x {
                        f.name == name
                    } else {
                        false
                    }
                });
                let main_export = match result {
                    Some(WasmExportView::Function(f)) => f,
                    _ => {
                        let e = "could not find export";
                        return Err(e);
                    }
                };
                Ok(main_export)
            } else {
                Err("could not find code section")
            }
        } else {
            Err("could not find export section")
        }
    }

    pub fn find_code_block<'a>(&'a self, index: usize) -> Result<&'a CodeBlock, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let SectionView::Code(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(SectionView::Code(code_section)) = result {
            if index >= code_section.code_blocks.len() {
                Err("invalid code block index")
            } else {
                Ok(&code_section.code_blocks[index])
            }
        } else {
            Err("could find code section")
        }
    }

    pub fn to_owned(&self) -> Program {
        Program {
            sections: self
                .sections
                .iter()
                .map(|x| x.to_owned())
                .collect::<Vec<Section>>(),
        }
    }
}

impl Program {
    pub fn find_exported_function<'a>(
        &'a self,
        name: &str,
    ) -> Result<&'a Export, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let Section::Export(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(Section::Export(export_section)) = result {
            let result = self.sections.iter().find(|x| {
                if let Section::Code(_) = x {
                    true
                } else {
                    false
                }
            });
            if let Some(Section::Code(_)) = result {
                let result = export_section.exports.iter().find(|x| {
                    if let WasmExport::Function(f) = x {
                        f.name == name
                    } else {
                        false
                    }
                });
                let main_export = match result {
                    Some(WasmExport::Function(f)) => f,
                    _ => {
                        let e = "could not find export";
                        return Err(e);
                    }
                };
                Ok(main_export)
            } else {
                Err("could not find code section")
            }
        } else {
            Err("could not find export section")
        }
    }

    pub fn find_code_block<'a>(&'a self, index: usize) -> Result<&'a CodeBlock, &'static str> {
        let result = self.sections.iter().find(|x| {
            if let Section::Code(_) = x {
                true
            } else {
                false
            }
        });
        if let Some(Section::Code(code_section)) = result {
            if index >= code_section.code_blocks.len() {
                Err("invalid code block index")
            } else {
                Ok(&code_section.code_blocks[index])
            }
        } else {
            Err("could find code section")
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut program_bytes = vec![];
        program_bytes.extend(MAGIC_NUMBER);
        program_bytes.extend(VERSION_1);
        for s in self.sections.iter() {
            match s {
                Section::Type(s) => {},
                Section::Function(s) => {},
                Section::Code(s) => {},
                Section::Export(s) => {},
                Section::Import(s) => {},
                Section::Memory(s) => {},
                Section::Start(s) => {
                    let mut sec_data = vec![];
                    sec_data.extend(s.start_function.to_wasm_bytes());
                    program_bytes.push(SECTION_START);
                    program_bytes.extend(sec_data.len().to_wasm_bytes());
                    program_bytes.extend(sec_data);
                },
                Section::Global(s) => {},
                Section::Table(s) => {},
                Section::Data(s) => {},
                Section::Custom(s) => {},
                Section::Element(s) => {},
            }
        }
        program_bytes
    }
}

pub fn parse<'p>(input: &'p [u8]) -> Result<ProgramView<'p>, &'static str> {
    wasm_module(input)
}

/// # Safety
///
/// This is an attempt to make this library compatible with c eventually
#[no_mangle]
#[cfg(feature = "c_extern")]
pub unsafe fn c_parse_web_assembly(ptr_wasm_bytes: *mut u8, len:usize) -> Program {
    let wasm_bytes = Vec::from_raw_parts(ptr_wasm_bytes,len,len);
    wasm_module(&wasm_bytes).unwrap().to_owned()
}