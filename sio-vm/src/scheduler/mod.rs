#![no_std]
extern crate alloc;
use alloc::{vec::Vec, sync::Arc};
const MSGSIZE: usize = 10;
use log::*;
use hashbrown::HashMap;
use core::sync::atomic::{AtomicUsize, Ordering};
use async_channel::{unbounded, Receiver, Sender};
use smol::{Executor, future::yield_now};
static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);
static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(0);
type ThreadId = usize;
type ProcessId = usize;
type VariableId = usize;
#[derive(Debug, Clone)]
pub enum Value {
    Unbound(VariableId),
    Int(VariableId, usize),
}
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
    pub fn int(thread_id: ThreadId, id: VariableId, value: usize) -> Self {
        Self::SynchVar(thread_id, Value::Int(id, value))
    }
}
enum ProcessRole {
    Corporal,
    Major,
    Brigadier,
}
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
    em: Vec<Operation>,
}
impl<'a> Thread {
    pub fn new(
        thread_id: ThreadId,
        thread_to_process_sender: Sender<Operation>, 
        process_to_thread_receiver: Receiver<Operation>, 
        em: Vec<Operation>) -> Self {
        Self {
            thread_id,
            thread_to_process_sender,
            state: ThreadState::Running,
            process_to_thread_receiver,
            em,
        }
    }
    fn werbolg_exec(&mut self) -> Option<Operation> {
        if !self.em.is_empty() {
            Some(self.em.remove(0))
        } else {
            None
        }
    }
    async fn run(&mut self) {
        loop {
            if let Some(operation) = self.werbolg_exec() {
                match operation {
                    Operation::SynchVar(ref thread_id, ref value) => {
                        match value {
                            Value::Unbound(index) => {
                                //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                                self.state = ThreadState::Waiting(operation.clone());
                                self.thread_to_process_sender.send(operation.clone()).await;
                            },
                            Value::Int(index, value) => {
                                //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                                self.state = ThreadState::Running;
                                self.thread_to_process_sender.send(operation.clone()).await;
                            }
                        }
                    },
                    Operation::ThreadSpawn(_,_) => {
                        //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                        self.thread_to_process_sender.send(operation.clone()).await;
                    },
                    Operation::WaitNeeded(thread_id, variable_index) => {
                        //info!("thread {} sent to process: {:?}", thread.thread_id, operation.clone());
                        self.state = ThreadState::WaitNeeded(variable_index);
                        self.thread_to_process_sender.send(Operation::WaitNeeded(thread_id, variable_index)).await;
                    },
                    Operation::ThreadTerminate(thread_id) => {
                        self.thread_to_process_sender.send(Operation::ThreadTerminate(self.thread_id)).await;
                        break;
                    }
                }
            } else {
                // There are no more operations to execute.
                self.thread_to_process_sender.send(Operation::ThreadTerminate(self.thread_id)).await;
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
                                if let (Value::Unbound(unbound_id), Value::Int(bound_id, _)) = (unbound_value, bound_value) {
                                    if *unbound_id == bound_id {
                                        //info!("waiting thread {} found {:?} and will start running", thread.thread_id, bound_id);
                                        self.state = ThreadState::Running;
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            info!("break");
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
                            if let Operation::SynchVar(_, Value::Int(bound_index, _)) = bound {
                                if variable_index == bound_index {
                                    self.state = ThreadState::Running;
                                }
                            }
                        }
                        Err(e) => break,
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
    em: Vec<Operation>,
}
impl<'a> Process<'a> {
    pub fn new(executor: Arc<Executor<'a>>, em: Vec<Operation>) -> Self {
        let (thread_to_process_sender, thread_to_process_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
        let process_to_thread_senders = HashMap::<ThreadId, Sender<Operation>>::new();
        Self {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            executor,
            process_to_thread_senders,
            thread_to_process_sender,
            thread_to_process_receiver,
            waiting_threads: HashMap::<VariableId, Vec<ThreadId>>::new(),
            wait_needed_threads: HashMap::<VariableId, ThreadId>::new(),
            bound_variables: HashMap::<VariableId, Operation>::new(),
            em,
        }
    }
    pub async fn run(&mut self) {
        let (process_to_thread_sender, process_to_thread_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
        let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
        self.process_to_thread_senders.insert(thread_id, process_to_thread_sender);
        let mut thread = Thread::new(
            thread_id, 
            self.thread_to_process_sender.clone(), 
            process_to_thread_receiver, 
            self.em.clone());
        self.executor.spawn(async move { thread.run().await }).detach();
        loop {
            match self.thread_to_process_receiver.recv().await {
                Ok(operation) => {
                    match operation {
                        Operation::SynchVar(thread_id, ref value) => {
                            match value {
                                Value::Unbound(variable_index) => {
                                    info!("thread_id {} with {:?}",thread_id, operation);
                                    // if already bound, notify waiting thread the value is bound
                                    if let Some(bound_variable_index) = self.bound_variables.get(variable_index) {
                                        if let Some(sender) = self.process_to_thread_senders.get(&thread_id) {
                                            sender.send(operation.clone()).await;
                                        }
                                    } else {
                                        self.waiting_threads.entry(*variable_index).or_insert_with(Vec::new).push(thread_id);
                                    }
                                    // notify wait needed threads to start work, enabling laziness
                                    if let Some(wait_needed_thread_id) = self.wait_needed_threads.get(variable_index) {
                                        if let Some(sender) = self.process_to_thread_senders.get(wait_needed_thread_id) {
                                            sender.send(operation.clone()).await;
                                        }
                                        self.wait_needed_threads.remove(variable_index);
                                    }
                                },
                                Value::Int(variable_index, value) => {
                                    info!("thread_id {} with {:?}",thread_id, operation);
                                    self.bound_variables.insert(*variable_index, operation.clone());
                                    if let Some(notify_threads) = self.waiting_threads.get(variable_index) {
                                        for thread_id in notify_threads {
                                            if let Some(sender) = self.process_to_thread_senders.get(thread_id) {
                                                sender.send(operation.clone()).await;
                                            }
                                        }
                                        self.waiting_threads.remove(variable_index);
                                    }
                                }
                            }   
                        },
                        Operation::ThreadSpawn(thread_id, ref em) => {
                            info!("thread_id {} with {:?}",thread_id, operation);
                            let (process_to_thread_sender, process_to_thread_receiver): (Sender<Operation>, Receiver<Operation>) = unbounded();
                            let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
                            self.process_to_thread_senders.insert(thread_id, process_to_thread_sender);
                            let mut thread = Thread::new(
                                thread_id, 
                                self.thread_to_process_sender.clone(), 
                                process_to_thread_receiver, 
                                em.to_vec());
                            self.executor.spawn(async move { thread.run().await }).detach();
                        }
                        Operation::WaitNeeded(thread_id, variable_index) => {
                            info!("thread_id {} WaitNeeded({})",thread_id, variable_index);
                            self.wait_needed_threads.entry(variable_index).or_insert(thread_id);
                        }
                        Operation::ThreadTerminate(thread_id) => {
                            //info!("thread_id {} ThreadTerminate",thread_id);
                            self.process_to_thread_senders.remove(&thread_id);
                            if self.process_to_thread_senders.is_empty() {
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    info!("break");
                    // Channel was closed, which means all threads have terminated
                    break;
                }
            }
            // info!("     unbound_variables: {:?}", self.waiting_threads);
            // info!("     bound_variables: {:?}", self.bound_variables);
            // info!("     wait_needed_variables: {:?}", self.wait_needed_threads);
        }
    }
}
