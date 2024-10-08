use sio_compiler::nifs::brigadier::{
    BrigadierLiteral, 
    brigadier_literal_mapper as literal_mapper, brigadier_literal_to_value as literal_to_value
};
use sio_compiler::allocator::BrigadierAllocator as Alloc;
use sio_compiler::value::brigadier::BrigadierValue as Value;
use sio_compiler::BrigadierState;

use hashbrown::HashSet;
use werbolg_compile::{code_dump, compile, Environment, InstructionAddress};
use werbolg_core::{id::IdF, AbsPath, Ident, Module, Namespace};
use werbolg_exec::{ExecutionEnviron, ExecutionMachine, ExecutionParams, WAllocator, NIF, WerRefCount};
use werbolg_lang_common::{Report, ReportKind, Source};
use werbolg_lang_lispy;
use sio_frontend;
use std::error::Error;

use super::{Frontend, SioParams};

pub fn run_frontend(
    params: &SioParams,
    args: &[String],
) -> Result<(Source, Module), Box<dyn Error>> {
    if args.is_empty() {
        crate::help();
        return Err(format!("no file specified").into());
    }

    let path = std::path::PathBuf::from(&args[0]);
    let source = get_file(&path)?;

    let parsing_res = werbolg_lang_lispy::module(&source.file_unit);
    let module = match parsing_res {
        Err(es) => {
            for e in es.into_iter() {
                let report = Report::new(ReportKind::Error, format!("Parse Error: {:?}", e))
                    .lines_before(1)
                    .lines_after(1)
                    .highlight(e.location, format!("parse error here"));

                report_print(&source, report)?;
            }
            return Err(format!("parse error").into());
        }
        Ok(module) => module,
    };
    if params.dump_ir {
        std::println!("{:#?}", module);
    }
    Ok((source, module))
}

pub fn report_print(source: &Source, report: Report) -> Result<(), Box<dyn Error>> {
    let mut s = String::new();
    report.write(&source, &mut s)?;
    println!("{}", s);
    Ok(())
}

pub fn run_compile<A>(
    params: &SioParams,
    env: &mut Environment<NIF<A, BrigadierLiteral, BrigadierState, Value>, Value>,
    source: Source,
    module: Module,
) -> Result<werbolg_compile::CompilationUnit<BrigadierLiteral>, Box<dyn Error>> {
    let module_ns = Namespace::root().append(Ident::from("main"));
    let modules = vec![(module_ns.clone(), module)];

    let compilation_params = werbolg_compile::CompilationParams {
        literal_mapper: literal_mapper,
        sequence_constructor: None,
    };

    let exec_module = match compile(&compilation_params, modules, env) {
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

    if params.dump_instr {
        let mut out = String::new();
        code_dump(&mut out, &exec_module.code, &exec_module.funs).expect("writing to string work");
        println!("{}", out);
    }

    Ok(exec_module)
}

pub fn run_exec(
    params: &SioParams,
    ee: ExecutionEnviron<Alloc, BrigadierLiteral, BrigadierState, Value>,
    exec_module: werbolg_compile::CompilationUnit<BrigadierLiteral>,
) -> Result<(), Box<dyn Error>> {
    let module_ns = Namespace::root().append(Ident::from("main"));

    let entry_point = exec_module
        .funs_tbl
        .get(&AbsPath::new(&module_ns, &Ident::from("main")))
        .expect("existing function as entry point");

    let execution_params = ExecutionParams {
        literal_to_value: literal_to_value,
    };
    let mut state = BrigadierState {};
    let mut allocator = Alloc {};

    let mut em = ExecutionMachine::new(
        WerRefCount::new(exec_module),
        WerRefCount::new(ee), execution_params, allocator, state);

    let mut stepper = HashSet::<InstructionAddress>::new();
    for a in params.step_address.iter() {
        stepper.insert(InstructionAddress::from_collection_len(*a as usize));
    }

    let ret = if !stepper.is_empty() | params.exec_step_trace {
        werbolg_exec::initialize(&mut em, entry_point, &[]).unwrap();
        loop {
            if params.exec_step_trace || stepper.contains(&em.ip) {
                let mut out = String::new();
                em.debug_state(&mut out).unwrap();
                println!("{}", out);
            }
            match werbolg_exec::step(&mut em) {
                Err(e) => break Err(e),
                Ok(None) => {}
                Ok(Some(v)) => break Ok(v),
            }
        }
    } else {
        werbolg_exec::exec(&mut em, entry_point, &[])
    };

    match ret {
        Err(e) => {
            let mut out = String::new();
            em.debug_state(&mut out).unwrap();

            println!("error: {:?} at {}", e, em.ip);
            println!("{}", out);
            return Err(format!("execution error").into());
        }
        Ok(val) => {
            println!("{:?}", val);
            Ok(())
        }
    }
}

fn get_file(path: &std::path::Path) -> std::io::Result<Source> {
    let path = std::path::PathBuf::from(&path);
    let content = std::fs::read_to_string(&path).expect("file read");
    let fileunit = Source::from_string(path.to_string_lossy().to_string(), content);
    Ok(fileunit)
}
