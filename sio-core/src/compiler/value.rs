use werbolg_core::{ConstrId, ValueFun};
use werbolg_exec::{ExecutionError, Valuable, ValueKind};

pub type ValueInt = u64;
pub type VariableId = u64;

#[derive(Clone, Debug)]
pub enum ThreadValue {
    Unit,
    Unbound(VariableId),
    Bool(VariableId, bool),
    Integer(VariableId, ValueInt),
    Fun(VariableId, ValueFun),
}

impl ThreadValue {
    fn desc(&self) -> ValueKind {
        match self {
            ThreadValue::Unit => UNIT_KIND,
            ThreadValue::Unbound(_) => UNBOUND_KIND,
            ThreadValue::Bool(_,_) => BOOL_KIND,
            ThreadValue::Integer(_,_) => INT_KIND,
            ThreadValue::Fun(_,_) => FUN_KIND,
        }
    }
}

pub const UNIT_KIND: ValueKind = "    unit";
pub const UNBOUND_KIND: ValueKind = " unbound";
pub const BOOL_KIND: ValueKind = "    bool";
pub const INT_KIND: ValueKind = "     int";
pub const FUN_KIND: ValueKind = "     fun";

impl Valuable for ThreadValue {
    fn descriptor(&self) -> werbolg_exec::ValueKind {
        self.desc()
    }

    fn conditional(&self) -> Option<bool> {
        match self {
            ThreadValue::Bool(i, b) => Some(*b),
            _ => None,
        }
    }

    fn fun(&self) -> Option<ValueFun> {
        match self {
            Self::Fun(variable_id, valuefun) => Some(*valuefun),
            _ => None,
        }
    }

    fn structure(&self) -> Option<(ConstrId, &[Self])> {
        None
    }

    fn index(&self, _index: usize) -> Option<&Self> {
        None
    }

    fn make_fun(fun: ValueFun) -> Self {
        ThreadValue::Fun(0, fun)
    }

    fn make_dummy() -> Self {
        ThreadValue::Unit
    }
}

impl ThreadValue {
    pub fn int(&self) -> Result<(VariableId, ValueInt), ExecutionError> {
        match self {
            ThreadValue::Integer(index, value) => Ok((*index, *value)),
            _ => Err(ExecutionError::ValueKindUnexpected {
                value_expected: INT_KIND,
                value_got: self.descriptor(),
            }),
        }
    }
}
