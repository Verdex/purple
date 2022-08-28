
use std::collections::HashMap;
use crate::error::VmError;

pub trait ToData<'a, T> where T : Clone + ToData<'a, T> {
    fn to_data(&self) -> Data<'a, T>;
}

#[derive(Debug, Clone)]
pub enum Data<'a, T : Clone + ToData<'a, T>> {
    Address(&'a T), // TODO probably becomes Address(Heap)
    Value(T),
    Func(Func),
}

#[derive(Debug, Clone, Copy)]
pub struct Func(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Label(pub usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Symbol(pub usize);

pub struct FuncDef<'a, T : Clone + ToData<'a, T>> {
    pub params : Vec<Label>,
    pub body : Vec<Instr<'a, T>>,
}

pub enum Instr<'a, T : Clone + ToData<'a, T>> { 
    Label(Label),
    Jump(Label),
    BranchOnTrue(Label, Box<dyn Fn(&Locals<'a, T>) -> Result<bool, Box<dyn std::error::Error>>>),
    Return(Symbol),
    LoadValue(Symbol, T),
    LoadFromReturn(Symbol),
    PushParam(Symbol),
    PopParam(Symbol),
    LoadFromExec(Symbol, Box<dyn Fn(&Locals<'a, T>) -> Result<Data<'a, T>, Box<dyn std::error::Error>>>),
    LoadFunc(Symbol, Func),
    Call(Symbol), // TODO can probably get rid of ToData if we insist that the symbol points to a func
                  // with the get from address at symbol we should be able to pull out funcs from heap
    Alloc { dest: Symbol, contents : Symbol }, 
    Free(Symbol),
    Store { address: Symbol, contents : Symbol },
    Get { address: Symbol, dest: Symbol },
}

#[derive(Debug, Clone)]
pub struct Locals<'a, T> where T : Clone + ToData<'a, T> {
    f : usize,
    v : HashMap<Symbol, Data<'a, T>>,
} 

impl<'a, T> Locals<'a, T> where T : Clone + ToData<'a, T> {
    pub fn new(func : usize) -> Self {
        Locals { v : HashMap::new(), f : func }
    }

    pub fn get(&self, sym : &Symbol) -> Result<Data<'a, T>, Box<dyn std::error::Error>> {
        match self.v.get(sym) {
            Some(x) => Ok(x.clone()),
            None => Err(Box::new(VmError::SymbolDoesNotExist { func : self.f, sym : sym.0 })),
        }
    }

    pub fn set(&mut self, sym : &Symbol, data : Data<'a, T>) -> Result<(), Box<dyn std::error::Error>> {
        self.v.insert(*sym, data);
        Ok(())
    }
}
