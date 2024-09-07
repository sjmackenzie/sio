

extern crate alloc;

pub mod allocator;
pub mod nifs;
pub mod value;

#[derive(Debug)]
pub enum CompilerError {
    UnsupportedFeature,
    SyntaxError,
    TypeError,
    OutOfMemory,
    InvalidArgument,
    UnresolvedReference,
    InvalidOperation,
    Overflow,
    Underflow,
    DivideByZero,
    Other,
}

pub use self::{
    allocator::{ThreadAllocator},
    value::{ThreadValue},
    nifs::{ThreadLiteral, thread_literal_mapper, thread_literal_to_value, create_thread_env},
};

pub type ThreadNIF = werbolg_exec::NIF<ThreadAllocator, ThreadLiteral, RunningThreadState, ThreadValue>;
pub type ThreadEnvironment = werbolg_compile::Environment<ThreadNIF, ThreadValue>;
pub type ThreadExecutionMachine =
    werbolg_exec::ExecutionMachine<ThreadAllocator, ThreadLiteral, RunningThreadState, ThreadValue>;

#[derive(Clone)]
pub struct RunningThreadState {}