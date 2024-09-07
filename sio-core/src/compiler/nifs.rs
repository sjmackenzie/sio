use crate::compiler::value::{ThreadValue as Value, ValueInt, VariableId};
use werbolg_compile::{CompilationError, Environment, CallArity};
use werbolg_core::{AbsPath, Ident, Literal, Namespace, Span};
use werbolg_exec::{ExecutionError, NIFCall, WAllocator};
use crate::compiler::{ThreadExecutionMachine, ThreadNIF};
use alloc::string::ToString;

fn nif_unbound(em: &mut ThreadExecutionMachine) -> Result<Value, ExecutionError> {
    let (_, args) = em.stack.get_call_and_args(em.current_arity);
    if args.is_empty() {
        Ok(Value::Unit)
    } else {
        Err(ExecutionError::UserPanic {
            message: "`nil' function does not need any arguments".to_string(),
        })
    }
}
fn nif_plus<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;
    let (_, n2) = args[1].int()?;

    let ret = Value::Integer(i1, n1 + n2);

    Ok(ret)
}

fn nif_sub<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;
    let (_, n2) = args[1].int()?;

    let ret = Value::Integer(i1, n1 - n2);

    Ok(ret)
}

fn nif_mul<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;
    let (_, n2) = args[1].int()?;

    let ret = Value::Integer(i1, n1 * n2);

    Ok(ret)
}

fn nif_neg<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;

    let ret = !n1;

    Ok(Value::Integer(i1, ret))
}

fn nif_eq<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;
    let (_, n2) = args[1].int()?;

    let ret = n1 == n2;

    Ok(Value::Bool(i1, ret))
}

fn nif_le<A: WAllocator>(_: &A, args: &[Value]) -> Result<Value, ExecutionError> {
    let (i1, n1) = args[0].int()?;
    let (_, n2) = args[1].int()?;

    let ret = n1 <= n2;

    Ok(Value::Bool(i1, ret))
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ThreadLiteral {
    Bool(VariableId, bool),
    Integer(VariableId, ValueInt),
}

pub fn thread_literal_to_value(lit: &ThreadLiteral) -> Value {
    match lit {
        ThreadLiteral::Bool(variable_id, b) => Value::Bool(*variable_id, *b),
        ThreadLiteral::Integer(variable_id, n) => Value::Integer(*variable_id, *n),
    }
}

// only support bool and number from the werbolg core literal
pub fn thread_literal_mapper(span: Span, lit: Literal) -> Result<ThreadLiteral, CompilationError> {
    match lit {
        Literal::Bool(b) => {
            let b = b.as_ref() == "true";
            Ok(ThreadLiteral::Bool(0, b))
        }
        Literal::Number(s) => {
            let Ok(v) = ValueInt::from_str_radix(s.as_ref(), 10) else {
                todo!()
            };
            Ok(ThreadLiteral::Integer(0, v))
        }
        Literal::String(_) => Err(CompilationError::LiteralNotSupported(span, lit)),
        Literal::Decimal(_) => Err(CompilationError::LiteralNotSupported(span, lit)),
        Literal::Bytes(_) => Err(CompilationError::LiteralNotSupported(span, lit)),
    }
}

pub fn create_thread_env(
) -> Environment<ThreadNIF, Value> {
    macro_rules! add_raw_nif {
        ($env:ident, $i:literal, $arity:literal, $e:expr) => {
            let nif = NIFCall::Raw($e).info($i, CallArity::try_from($arity as usize).unwrap());
            let path = AbsPath::new(&Namespace::root(), &Ident::from($i));
            let _ = $env.add_nif(&path, nif);
        };
    }
    macro_rules! add_pure_nif {
        ($env:ident, $i:literal, $arity:literal, $e:expr) => {
            let nif = NIFCall::Pure($e).info($i, CallArity::try_from($arity as usize).unwrap());
            let path = AbsPath::new(&Namespace::root(), &Ident::from($i));
            let _ = $env.add_nif(&path, nif);
        };
    }
    let mut env = Environment::new();
    add_raw_nif!(env, "unbound", 0, nif_unbound);
    add_pure_nif!(env, "+", 2, nif_plus);
    add_pure_nif!(env, "-", 2, nif_sub);
    add_pure_nif!(env, "*", 2, nif_mul);
    add_pure_nif!(env, "==", 2, nif_eq);
    add_pure_nif!(env, "<=", 2, nif_le);
    add_pure_nif!(env, "neg", 1, nif_neg);
    env
}
