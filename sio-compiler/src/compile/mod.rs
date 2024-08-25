use crate::{
    CorporalExecutionMachine, CorporalEnvironment, CorporalAllocator, CorporalLiteral, CorporalState, CorporalValue, corporal_literal_mapper, corporal_literal_to_value
};
use werbolg_core::{AbsPath, Ident, Namespace, ir::Module};
use werbolg_exec::{ ExecutionMachine, ExecutionEnviron, ExecutionParams, WerRefCount};
use werbolg_compile::{compile};
use werbolg_lang_common::{Report, ReportKind, Source};
use alloc::{format, vec, vec::Vec, boxed::Box, string::String};

use core::error::Error;


fn compile_module(
    //params: SioParams,
    env: &mut CorporalEnvironment,
    source: Source,
    module: Module,
) -> Result<werbolg_compile::CompilationUnit<CorporalLiteral>, Box<dyn Error>> {
    //let (source, module) = run_frontend(src, path).unwrap();
    let module_ns = Namespace::root().append(Ident::from("main"));
    let modules = vec![(module_ns.clone(), module)];
    let compilation_params = werbolg_compile::CompilationParams {
        literal_mapper: corporal_literal_mapper,
        sequence_constructor: None,
    };
    let cu = match compile(&compilation_params, modules, env) {
        Err(e) => {
            let report = Report::new(ReportKind::Error, format!("Compilation Error: {:?}", e))
                .lines_before(1)
                .lines_after(1)
                .highlight(e.span().unwrap(), format!("compilation error here"));
            report_print(&source, report)?;
            return Err(format!("compilation error {:?}", e).into());
        }
        Ok(m) => m,
    };
    //if params.dump_instr {
    //    let mut out = String::new();
    //    code_dump(&mut out, &cu.code, &cu.funs).expect("writing to string work");
        //println!("{}", out);
    //}
    Ok(cu)
}

pub fn build_machine (
    ee: ExecutionEnviron<CorporalAllocator, CorporalLiteral, CorporalState, CorporalValue>,
    cu: werbolg_compile::CompilationUnit<CorporalLiteral>,
) -> Result<CorporalExecutionMachine, Box<dyn Error>> {
    let module_ns = Namespace::root().append(Ident::from("main"));
    let entry_point = cu
        .funs_tbl
        .get(&AbsPath::new(&module_ns, &Ident::from("main")))
        .expect("existing function as entry point");
    let execution_params = ExecutionParams {
        literal_to_value: corporal_literal_to_value,
    };
    let state = CorporalState {};
    let allocator = CorporalAllocator {};
    let mut em = ExecutionMachine::new(
        WerRefCount::new(cu),
        WerRefCount::new(ee),
        execution_params, allocator, state);
    werbolg_exec::initialize(&mut em, entry_point, &[]).unwrap();
    Ok(em)
}

pub fn make_execution_machine(
    src: String, 
    path: String, 
    mut env: CorporalEnvironment
) -> Result<Vec<CorporalExecutionMachine>, Box<dyn Error>> {
    let (source, module) = run_frontend(src, path)?;
    let cu = compile_corporal(&mut env, source, module)?;
    let ee = werbolg_exec::ExecutionEnviron::from_compile_environment(env.finalize());
    let em = build_corporal_machine(ee, cu)?;
    Ok(vec![em])
}
