use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use watson::*;

fn value_type_to_string(v: &ValueType) -> String {
    match v {
        ValueType::I32 => "I32".to_string(),
        ValueType::I64 => "I64".to_string(),
        ValueType::F32 => "F32".to_string(),
        ValueType::F64 => "F64".to_string(),
    }
}

fn print_type_section(s: &TypeSection) {
    println!("  [{}]", "Type".purple());
    for i in 0..s.types.len() {
        match &s.types[i] {
            WasmType::Function(f) => {
                println!(
                    "  {}: fn(inputs{:?}) -> outputs{:?}",
                    i,
                    f.inputs
                        .iter()
                        .map(|x| value_type_to_string(x))
                        .collect::<Vec<String>>(),
                    f.outputs
                        .iter()
                        .map(|x| value_type_to_string(x))
                        .collect::<Vec<String>>()
                );
            }
        }
    }
}

fn print_function_section(s: &FunctionSection) {
    println!("  [{}]", "Function".purple());
    for i in 0..s.function_types.len() {
        println!("  {}: type[{:?}]", i, s.function_types[i]);
    }
}

fn print_export_section(s: &ExportSection) {
    println!("  [{}]", "Export".purple());
    for i in 0..s.exports.len() {
        match &s.exports[i] {
            WasmExport::Function(f) => {
                println!("  {:?} function[{}]", f.name, f.index);
            }
            WasmExport::Memory(f) => {
                println!("  {:?} memory[{}]", f.name, f.index);
            }
            WasmExport::Global(f) => {
                println!("  {:?} global[{}]", f.name, f.index);
            }

            WasmExport::Table(f) => {
                println!("  {:?} table[{}]", f.name, f.index);
            }
        }
    }
}

fn print_import_section(s: &ImportSection) {
    println!("  [{}]", "Import".purple());
    for i in 0..s.imports.len() {
        match &s.imports[i] {
            WasmImport::Function(f) => {
                println!(
                    "  {:?}.{:?} fn type[{}]",
                    f.module_name, f.name, f.type_index
                );
            }
            WasmImport::Memory(f) => {
                if f.max_pages.is_some() {
                    println!(
                        "  {:?}.{:?} memory min {} max {}",
                        f.module_name,
                        f.name,
                        f.min_pages,
                        f.max_pages.unwrap()
                    );
                } else {
                    println!(
                        "  {:?}.{:?} memory min {}",
                        f.module_name, f.name, f.min_pages
                    );
                }
            }
            WasmImport::Table(f) => {
                if f.max.is_some() {
                    println!(
                        "  {:?}.{:?} table \"ANYFUNC\" min {} max {}",
                        f.module_name,
                        f.name,
                        f.min,
                        f.max.unwrap()
                    );
                } else {
                    println!(
                        "  {:?}.{:?} table \"ANYFUNC\" min {}",
                        f.module_name, f.name, f.min
                    );
                }
            }
            WasmImport::Global(f) => {
                if f.is_mutable {
                    println!(
                        "  {:?}.{:?} global mut {}",
                        f.module_name,
                        f.name,
                        value_type_to_string(&f.value_type)
                    );
                } else {
                    println!(
                        "  {:?}.{:?} iglobal mm {}",
                        f.module_name,
                        f.name,
                        value_type_to_string(&f.value_type)
                    );
                }
            }
        }
    }
}

fn print_table_section(s: &TableSection) {
    println!("  [{}]", "Table".purple());
    for (i, t) in s.tables.iter().enumerate() {
        if t.max.is_some() {
            println!(
                "  {:?}: \"ANYREF\" min {:?} max {:?}",
                i,
                t.min,
                t.max.unwrap()
            );
        } else {
            println!("  {:?}: \"ANYREF\" min {:?}", i, t.min);
        }
    }
}

fn print_global_section(s: &GlobalSection) {
    println!("  [{}]", "Global".purple());
    for i in 0..s.globals.len() {
        let g = &s.globals[i];
        if g.is_mutable {
            println!(
                "  {}: mut {:?} expr: {:?}",
                i,
                value_type_to_string(&g.value_type),
                g.expression
            );
        } else {
            println!(
                "  {}: imm {:?} expr: {:?}",
                i,
                value_type_to_string(&g.value_type),
                g.expression
            );
        }
    }
}

fn print_memory_section(s: &MemorySection) {
    println!("  [{}]", "Memory".purple());
    for i in 0..s.memories.len() {
        println!(
            "  {}: min {} max {}",
            i,
            s.memories[i].min_pages,
            &match s.memories[i].max_pages {
                Some(m) => m.to_string(),
                None => "".to_string(),
            }
        );
    }
}

fn print_code_section(s: &CodeSection) {
    println!("  [{}]", "Code".purple());
    for i in 0..s.code_blocks.len() {
        println!(
            "  {}: locals{:?} code{:?}",
            i,
            s.code_blocks[i]
                .locals
                .iter()
                .map(|x| (x.0, value_type_to_string(&x.1)))
                .collect::<Vec<(u32, String)>>(),
            s.code_blocks[i].code
        );
    }
}

fn print_data_section(s: &DataSection) {
    println!("  [{}]", "Data".purple());
    for (i, d) in s.data_blocks.iter().enumerate() {
        println!(
            "  {}: memory[{:?}] offset_expression{:?} data{:?}",
            i, d.memory, d.offset_expression, d.data,
        );
    }
}

fn print_element_section(s: &ElementSection) {
    println!("  [{}]", "Element".purple());
    for (i, d) in s.elements.iter().enumerate() {
        println!(
            "  {}: table[{:?}] expression{:?} functions{:?}",
            i, d.table, d.expression, d.functions,
        );
    }
}

fn print_custom_section(s: &CustomSection) {
    println!("  [{}]", "Custom".purple());
    println!("  {}  data{:?}", s.name, s.data,);
}

fn print_unknown_section(s: &UnknownSection) {
    println!("  [{}:{}]", "Unknown".purple(), s.id);
    println!("  {:?}", s.data);
}

fn print_start_section(s: &StartSection) {
    println!("  [{}]", "Start".purple());
    println!("  {:?}", s.start_function);
}

fn print_section(s: &Section) {
    match s {
        Section::Type(s) => print_type_section(&s),
        Section::Function(s) => print_function_section(&s),
        Section::Export(s) => print_export_section(&s),
        Section::Code(s) => print_code_section(&s),
        Section::Memory(s) => print_memory_section(&s),
        Section::Unknown(s) => print_unknown_section(&s),
        Section::Start(s) => print_start_section(&s),
        Section::Import(s) => print_import_section(&s),
        Section::Table(s) => print_table_section(&s),
        Section::Global(s) => print_global_section(&s),
        Section::Data(s) => print_data_section(&s),
        Section::Custom(s) => print_custom_section(&s),
        Section::Element(s) => print_element_section(&s),
    }
}

fn print_program(name: &str, program: &Program) {
    println!("{} {{", &name.green());
    for s in program.sections.iter() {
        print_section(&s);
    }
    println!("}}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("first arg should be a file");
        return Ok(());
    }
    let mut f = File::open(&args[1])?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    match Program::parse(&buffer) {
        Ok(p) => print_program(&args[1], &p),
        Err(e) => {
            print_program(&args[1], &e.0);
            println!("{}", e.1.red());
        }
    };
    Ok(())
}
