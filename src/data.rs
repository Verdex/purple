
use crate::error::VmError;

pub trait ToData<'a, T> where T : Clone + ToData<'a, T> {
    fn to_data(&self) -> Data<'a, T>;
}

#[derive(Debug, Clone)]
pub enum Data<'a, T : Clone + ToData<'a, T>> {
    Address(&'a T),
    Value(T),
    Func(Func),
}

#[derive(Debug, Clone, Copy)]
pub struct Func(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Label(pub usize);
#[derive(Debug, Clone, Copy)]
pub struct Symbol(pub usize);

pub struct FuncDef<'a, T : Clone + ToData<'a, T>> {
    pub params : Vec<Label>,
    pub body : Vec<Instr<'a, T>>,
}

pub enum Instr<'a, T : Clone + ToData<'a, T>> { 
    Label(Label),
    Jump(Label),
    BranchOnTrue(Label, Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<bool, Box<dyn std::error::Error>>>),
    Return(Symbol),
    LoadValue(Symbol, T),
    LoadFromReturn(Symbol),
    PushParam(Symbol),
    LoadFromExec(Symbol, Box<dyn FnMut(&Locals<'a, T>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
    LoadFunc(Symbol, Func),
    Call(Symbol),
    SysCall(Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<(), Box<dyn std::error::Error>>>),
    LoadFromSysCall(Symbol, Box<dyn FnMut(&Locals<'a, T>, &'a Vec<Data<'a, T>>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
}

#[derive(Debug, Clone)]
pub struct Locals<'a, T> where T : Clone + ToData<'a, T> {
    f : usize,
    v : Vec<Data<'a, T>>,
} 

impl<'a, T> Locals<'a, T> where T : Clone + ToData<'a, T> {
    pub fn new(func : usize) -> Self {
        Locals { v : vec![], f : func }
    }

    pub fn get(&self, sym : &Symbol) -> Result<Data<'a, T>, Box<dyn std::error::Error>> {
        if self.v.len() <= sym.0 {
            Err(Box::new(VmError::SymbolDoesNotExist { func : self.f, sym : sym.0 }))
        }
        else {
            Ok(self.v[sym.0].clone())
        }
    }
}
