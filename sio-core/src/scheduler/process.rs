use alloc::{format, vec, vec::Vec, boxed::Box, string::String, sync::Arc};
const MSGSIZE: usize = 10;
//use log::*;
use hashbrown::HashMap;
use core::sync::atomic::{AtomicU64, Ordering};
use async_channel::{unbounded, Receiver, Sender};
use smol::{Executor};
use werbolg_exec::{
    ExecutionEnviron, 
    //ExecutionError, 
    ExecutionMachine, ExecutionParams, WerRefCount, step
};
use crate::frontend;
use crate::compiler::{
    //process::run_frontend,
    ThreadExecutionMachine, ThreadEnvironment, ThreadAllocator, ThreadLiteral, RunningThreadState, ThreadValue as Value, thread_literal_mapper, thread_literal_to_value
};
use werbolg_core::{AbsPath, Ident, Namespace, ir::Module};
use werbolg_compile::{compile};
use werbolg_lang_common::{Report, ReportKind, Source};
use crate::compiler::create_thread_env;
use crate::compiler::value::VariableId;
use core::error::Error;
//use log::info;
static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(0);
type ThreadId = u64;
type ProcessId = u64;
#[derive(Debug, Clone)]
pub enum Operation {
    SynchVar(ThreadId, Value),
    ThreadSpawn(ThreadId, Vec<Operation>),
    WaitNeeded(ThreadId, VariableId),
    ThreadTerminate(ThreadId),
    //Portcullis(ThreadId, Operation),
}
impl Operation {
    pub fn unbound(thread_id: ThreadId, id: VariableId) -> Self {
        Self::SynchVar(thread_id, Value::Unbound(id))
    }
    pub fn int(thread_id: ThreadId, id: VariableId, value: u64) -> Self {
        Self::SynchVar(thread_id, Value::Integer(id, value))
    }
}
enum ProcessRole {
    Corporal,
    Major,
    Brigadier,
}
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(0);
enum ThreadState {
    Running,
    Waiting(Operation),
    WaitNeeded(VariableId),
}
pub struct Thread {
    thread_id: ThreadId,
    thread_to_process_sender: Sender<Operation>,
    process_to_thread_receiver: Receiver<Operation>,
    state: ThreadState,
    em: ThreadExecutionMachine,
}
impl<'a> Thread {
    pub fn new(
        thread_id: ThreadId,
        thread_to_process_sender: Sender<Operation>, 
        process_to_thread_receiver: Receiver<Operation>, 
        em: ThreadExecutionMachine) -> Self {
        Self {
            thread_id,
            thread_to_process_sender,
            state: ThreadState::Running,
            process_to_thread_receiver,
            em,
        }
    }
    fn werbolg_exec(&mut self) -> Option<Operation> {
        match step(&mut self.em).ok()? {
            None => None,
            Some(v) => Some(Operation::SynchVar(self.thread_id, v)),
        }
    }
    async fn run(&mut self) {
        loop {
            if let Some(operation) = self.werbolg_exec() {
                match operation {
                    Operation::SynchVar(ref _thread_id, ref value) => {
                        match value {
                            Value::Unbound(_index) => {
                                //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                                self.state = ThreadState::Waiting(operation.clone());
                                let _ = self.thread_to_process_sender.send(operation.clone()).await;
                            },
                            Value::Integer(_index, _value) => {
                                //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                                self.state = ThreadState::Running;
                                let _ = self.thread_to_process_sender.send(operation.clone()).await;
                            }
                            &Value::Unit | &Value::Bool(_,_) | &Value::Fun(_,_) => todo!()
                        }
                    },
                    Operation::ThreadSpawn(_,_) => {
                        //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                        let _ = self.thread_to_process_sender.send(operation.clone()).await;
                    },
                    Operation::WaitNeeded(thread_id, variable_index) => {
                        //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                        self.state = ThreadState::WaitNeeded(variable_index);
                        let _ = self.thread_to_process_sender.send(Operation::WaitNeeded(thread_id, variable_index)).await;
                    },
                    Operation::ThreadTerminate(_thread_id) => {
                        let _ = self.thread_to_process_sender.send(Operation::ThreadTerminate(self.thread_id)).await;
                        break;
                    }
                }
            } else {
                // There are no more operations to execute.
                let _ = self.thread_to_process_sender.send(Operation::ThreadTerminate(self.thread_id)).await;
                break;
            }
            match self.state {
                ThreadState::Running => {
                },
                ThreadState::Waiting(ref unbound) => {
                    //info!("thread {} is waiting for unbound variable {:?}", thread.thread_id, unbound);
                    match self.process_to_thread_receiver.recv().await {
                        Ok(bound) => {
                            if let (Operation::SynchVar(_, unbound_value), Operation::SynchVar(_, bound_value)) = (unbound, bound) {
                                if let (Value::Unbound(unbound_id), Value::Integer(bound_id, _)) = (unbound_value, bound_value) {
                                    if *unbound_id == bound_id {
                                        //info!("waiting thread {} found {:?} and will start running", thread.thread_id, bound_id);
                                        self.state = ThreadState::Running;
                                    }
                                }
                            }
                        },
                        Err(_e) => {
                            //info!("break");
                            // Channel was closed, which means all threads have terminated
                            break;
                        }
                    }
                },
                ThreadState::WaitNeeded(variable_index) => {
                    //info!("thread {} is waiting on wait_needed variable_index {}", thread.thread_id, variable_index);
                    let operation = self.process_to_thread_receiver.recv().await;
                    match operation {
                        Ok(bound) => {
                            if let Operation::SynchVar(_, Value::Integer(bound_index, _)) = bound {
                                if variable_index == bound_index {
                                    self.state = ThreadState::Running;
                                }
                            }
                        }
                        Err(_e) => break,
                    }
                }
            }
        }
    }
}
pub struct Process<'a> {
    process_id: ProcessId,
    executor: Arc<Executor<'a>>,
    //role: ProcessRole,
    process_to_thread_senders: HashMap<ThreadId, Sender<Operation>>,
    thread_to_process_sender: Sender<Operation>,
    thread_to_process_receiver: Receiver<Operation>,
    wait_needed_threads: HashMap<VariableId, ThreadId>,
    waiting_threads: HashMap<VariableId, Vec<ThreadId>>,
    bound_variables: HashMap<VariableId, Operation>,
    source: Arc<Source>,
    module: Module,
    env: ThreadEnvironment,
    //em: Vec<Operation>,
}
impl<'a> Process<'a> {
    pub fn new(
        executor: Arc<Executor<'a>>, 
        //em: Vec<Operation>,
        src: String,
        path: String,
    ) -> Result<Self, Box<dyn Error>> {
        let (thread_to_process_sender, thread_to_process_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
        let process_to_thread_senders = HashMap::<ThreadId, Sender<Operation>>::new();
        let env = create_thread_env();
        let (source, module) = run_frontend(src, path)?;
        let source = Arc::new(source);
        Ok(Self {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            executor,
            process_to_thread_senders,
            thread_to_process_sender,
            thread_to_process_receiver,
            waiting_threads: HashMap::<VariableId, Vec<ThreadId>>::new(),
            wait_needed_threads: HashMap::<VariableId, ThreadId>::new(),
            bound_variables: HashMap::<VariableId, Operation>::new(),
            source, 
            module, 
            env
        })
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let (process_to_thread_sender, process_to_thread_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
        let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
        self.process_to_thread_senders.insert(thread_id, process_to_thread_sender);
        let mut env = create_thread_env();
        let cu = compile_thread(&mut env, self.source.clone(), self.module.clone())?;
        let ee = werbolg_exec::ExecutionEnviron::from_compile_environment(env.finalize());
        let em = build_thread_machine(ee, cu)?;
        let mut thread = Thread::new(
            thread_id, 
            self.thread_to_process_sender.clone(), 
            process_to_thread_receiver, 
            em);
        self.executor.spawn(async move { thread.run().await }).detach();
        loop {
            match self.thread_to_process_receiver.recv().await {
                Ok(operation) => {
                    match operation {
                        Operation::SynchVar(thread_id, ref value) => {
                            match value {
                                Value::Unbound(variable_index) => {
                                    //info!("thread_id {} with {:?}",thread_id, operation);
                                    // if already bound, notify waiting thread the value is bound
                                    if let Some(_bound_variable_index) = self.bound_variables.get(variable_index) {
                                        if let Some(sender) = self.process_to_thread_senders.get(&thread_id) {
                                            let _ = sender.send(operation.clone()).await;
                                        }
                                    } else {
                                        self.waiting_threads.entry(*variable_index).or_insert_with(Vec::new).push(thread_id);
                                    }
                                    // notify wait needed threads to start work, enabling laziness
                                    if let Some(wait_needed_thread_id) = self.wait_needed_threads.get(variable_index) {
                                        if let Some(sender) = self.process_to_thread_senders.get(wait_needed_thread_id) {
                                            let _ = sender.send(operation.clone()).await;
                                        }
                                        self.wait_needed_threads.remove(variable_index);
                                    }
                                },
                                Value::Integer(variable_index, _value) => {
                                    //info!("thread_id {} with {:?}",thread_id, operation);
                                    self.bound_variables.insert(*variable_index, operation.clone());
                                    if let Some(notify_threads) = self.waiting_threads.get(variable_index) {
                                        for thread_id in notify_threads {
                                            if let Some(sender) = self.process_to_thread_senders.get(thread_id) {
                                                let _ = sender.send(operation.clone()).await;
                                            }
                                        }
                                        self.waiting_threads.remove(variable_index);
                                    }
                                },
                                &Value::Unit | &Value::Bool(_, _) | &Value::Fun(_, _) => todo!()
                            }   
                        },
                        Operation::ThreadSpawn(_thread_id, ref _em) => {
                            //info!("thread_id {} with {:?}",thread_id, operation);
                            let (process_to_thread_sender, process_to_thread_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
                            let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
                            self.process_to_thread_senders.insert(thread_id, process_to_thread_sender);
                            let env = create_thread_env();
                            let cu = compile_thread(&mut self.env, self.source.clone(), self.module.clone())?;
                            let ee = werbolg_exec::ExecutionEnviron::from_compile_environment(env.finalize());
                            let em = build_thread_machine(ee, cu)?;
                            let mut thread = Thread::new(
                                thread_id, 
                                self.thread_to_process_sender.clone(), 
                                process_to_thread_receiver, 
                                em);
                            self.executor.spawn(async move { thread.run().await }).detach();
                        }
                        Operation::WaitNeeded(thread_id, variable_index) => {
                            //info!("thread_id {} WaitNeeded({})",thread_id, variable_index);
                            self.wait_needed_threads.entry(variable_index).or_insert(thread_id);
                        }
                        Operation::ThreadTerminate(thread_id) => {
                            //info!("thread_id {} ThreadTerminate",thread_id);
                            self.process_to_thread_senders.remove(&thread_id);
                            if self.process_to_thread_senders.is_empty() {
                                break Ok(());
                            }
                        }
                    }
                },
                Err(_e) => {
                    //info!("break");
                    // Channel was closed, which means all threads have terminated
                    break Ok(());
                }
            }
            // info!("     unbound_variables: {:?}", self.waiting_threads);
            // info!("     bound_variables: {:?}", self.bound_variables);
            // info!("     wait_needed_variables: {:?}", self.wait_needed_threads);
        }
    }
}


fn compile_thread(
    //params: SioParams,
    env: &mut ThreadEnvironment,
    source: Arc<Source>,
    module: Module,
) -> Result<werbolg_compile::CompilationUnit<ThreadLiteral>, Box<dyn Error>> {
    //let (source, module) = run_frontend(src, path).unwrap();
    let module_ns = Namespace::root().append(Ident::from("main"));
    let modules = vec![(module_ns.clone(), module)];
    let compilation_params = werbolg_compile::CompilationParams {
        literal_mapper: thread_literal_mapper,
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

pub fn build_thread_machine (
    ee: ExecutionEnviron<ThreadAllocator, ThreadLiteral, RunningThreadState, Value>,
    cu: werbolg_compile::CompilationUnit<ThreadLiteral>,
) -> Result<ThreadExecutionMachine, Box<dyn Error>> {
    let module_ns = Namespace::root().append(Ident::from("main"));
    let entry_point = cu
        .funs_tbl
        .get(&AbsPath::new(&module_ns, &Ident::from("main")))
        .expect("existing function as entry point");
    let execution_params = ExecutionParams {
        literal_to_value: thread_literal_to_value,
    };
    let state = RunningThreadState {};
    let allocator = ThreadAllocator {};
    let mut em = ExecutionMachine::new(
        WerRefCount::new(cu),
        WerRefCount::new(ee),
        execution_params, allocator, state);
    werbolg_exec::initialize(&mut em, entry_point, &[]).unwrap();
    Ok(em)
}

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

                report_print(&source, report)?;
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

#[cfg(test)]
mod tests {
    //use alloc::vec::Vec;
    //use alloc::vec;
    use alloc::string::ToString;
    use sio::compiler::create_thread_env;
    use super::*;
    static src: &str =
        "
        url public_key : sio79f708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abd;
        url type : src;
        url name : app_name;
        url app : public_key::type::name;
        corporal app::Corporal {
            pub master :: () {
                let x, y;
                let z;
                thread {
                    let assign_x = lazy (x) {
                        x = 0;
                    };
                    assign_x(x);
                }
                thread {
                    assign_y(y);
                }
                if x == y {
                    z = 1;
                }
            }
            lazy assign_y :: (y) {
                y = 0;
            }
        }";
    #[test]
    fn basic_lazy_concurrent_dataflow() {
        let env = create_thread_env();
        use sio_vm_std::run_frontend;
        let ex = Arc::new(Executor::new());
        //let (source, module) = run_frontend(src, path)?;
        let mut process = Process::new(src.to_string(), "/".to_string(), env).expect("Corporal failure reason:");
        smol::block_on(ex.run(process.run()));
        assert_eq!(4, 4);
        Ok(())
    }
}