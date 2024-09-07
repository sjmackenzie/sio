#![no_std]
#![feature(error_in_core)]
extern crate alloc;
use alloc::format;
use alloc::string::String;
use werbolg_lang_common::Source;
use werbolg_core::Module;
use alloc::boxed::Box;
use core::error::Error;
use werbolg_lang_common::Report;
use werbolg_lang_common::ReportKind;
use sio_core::frontend;
//use log::info;

//pub mod process;
//mod frontend;
//pub mod compiler;

pub fn run_frontend(src: String, path: String) -> Result<(Source, Module), Box<dyn Error>> {
    let source = Source::from_string(path, src);
    let parsing_res = frontend::module(&source.file_unit);
    let module = match parsing_res {
        Err(es) => {
            for e in es.into_iter() {
                let report = Report::new(ReportKind::Error, format!("Parse Error: {:?}", e.message))
                    .lines_before(1)
                    .lines_after(1)
                    .highlight(e.span.start.0 as usize .. e.span.end.0 as usize, e.message);

                //report_print(&source, report)?;
            }
            return Err(format!("parse error").into());
        }
        Ok(module) => module,
    };
    Ok((source, module))
} 

pub fn report_print(source: &Source, report: Report) -> Result<(), Box<dyn Error>> {
    let mut s = String::new();
    report.write(&source, &mut s)?;
    //info!("{}", s);
    Ok(())
}