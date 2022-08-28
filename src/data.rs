
use std::collections::HashMap;
use crate::error::VmError;

#[derive(Debug, Clone)]
pub enum Data<T : Clone> {
    Address(Address), 
    Value(T),
    Func(Func),
}

#[derive(Debug, Clone, Copy)]
pub struct Func(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Label(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Address(pub usize);

pub struct FuncDef<T : Clone> {
    pub params : Vec<Label>,
    pub body : Vec<Instr<T>>,
}

pub enum Instr<T : Clone> { 
    Label(Label),
    Jump(Label),
    BranchOnTrue(Label, Box<dyn Fn(&Locals<T>) -> Result<bool, Box<dyn std::error::Error>>>),
    Return(Symbol),
    LoadValue(Symbol, T),
    LoadFromReturn(Symbol),
    PushParam(Symbol),
    PopParam(Symbol),
    LoadFromExec(Symbol, Box<dyn Fn(&Locals<T>) -> Result<Data<T>, Box<dyn std::error::Error>>>),
    LoadFunc(Symbol, Func),
    Call(Symbol), 
    Alloc { dest: Symbol, contents : Symbol }, 
    Free(Symbol),
    Store { address: Symbol, contents : Symbol },
    Get { address: Symbol, dest: Symbol },
}

#[derive(Debug, Clone)]
pub struct Locals<T> where T : Clone {
    f : usize,
    v : HashMap<Symbol, Data<T>>,
} 

impl<T> Locals<T> where T : Clone {
    pub fn new(func : usize) -> Self {
        Locals { v : HashMap::new(), f : func }
    }

    pub fn get(&self, sym : &Symbol) -> Result<Data<T>, Box<dyn std::error::Error>> {
        match self.v.get(sym) {
            Some(x) => Ok(x.clone()),
            None => Err(Box::new(VmError::SymbolDoesNotExist { func : self.f, sym : sym.0 })),
        }
    }

    pub fn set(&mut self, sym : &Symbol, data : Data<T>) -> Result<(), Box<dyn std::error::Error>> {
        self.v.insert(*sym, data);
        Ok(())
    }
}
