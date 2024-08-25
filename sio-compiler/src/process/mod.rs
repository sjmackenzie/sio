use alloc::{
    format,
    string::String,
};
use werbolg_lang_common::{Source, Report, ReportKind};
use werbolg_core::Module;
use alloc::boxed::Box;
use core::error::Error;

pub mod general;
pub mod brigadier;
pub mod major;
pub mod corporal;

fn run_frontend(src: String, path: String) -> Result<(Source, Module), Box<dyn Error>> {
    let source = Source::from_string(path, src);
    let parsing_res = sio_frontend::module(&source.file_unit);
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