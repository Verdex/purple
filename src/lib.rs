
use std::collections::HashMap;

pub mod error;
pub mod data;

use crate::error::VmError;
use crate::data::*;

type R<T> = Result<T, Box<dyn std::error::Error>>;

struct Frame<T : Clone> {
    instr_ptr : usize,
    locals : Locals<T>,
    current_function : Func,
}

struct FuncDefWithLabel<'a, T : Clone, Env> {
    pub body : &'a Vec<Instr<T, Env>>,
    pub label_map : HashMap<Label, usize>,
}

pub fn run<T : Clone, Env>( func_defs : &HashMap<Func, Vec<Instr<T, Env>>>, env: &mut Env ) -> R<Option<Data<T>>> {

    let mut current_function : Func = Func(0);

    if !func_defs.contains_key(&current_function) {
        return Err(Box::new(VmError::FunctionDoesNotExist(0)));
    }

    let func_defs_with_label = func_defs.iter()
                                        .map(|kvp| Ok((*kvp.0, setup_label_map(kvp.1, *kvp.0)?)))
                                        .collect::<R<HashMap<_, _>>>()?;

    let mut stack : Vec<Frame<T>> = vec![];
    // NOTE:  We checked that Func(0) is present already.
    let mut instrs : &Vec<Instr<T, Env>> = &func_defs_with_label.get(&current_function).unwrap().body;
    let mut label_map : &HashMap<Label, usize> = &func_defs_with_label.get(&current_function).unwrap().label_map;
    let mut instr_ptr = 0;
    let mut locals : Locals<T> = Locals::new(current_function.0);
    let mut params : Vec<Data<T>> = vec![];
    let mut ret = None;

    loop {

        if instrs.len() <= instr_ptr {
            if stack.len() == 0 {
                break;
            }

            Frame { instr_ptr, locals, current_function } = stack.pop().unwrap();
            // NOTE:  We don't have to check if current_function exists because if we're poping
            // then we must have called it previously.
            instrs = &func_defs_with_label.get(&current_function).unwrap().body;
            label_map = &func_defs_with_label.get(&current_function).unwrap().label_map;
        }

        match &instrs[instr_ptr] {
            Instr::Label(_) => { instr_ptr += 1; },
            Instr::Jump(label) =>
                match label_map.get(label) {
                    Some(ptr) => instr_ptr = *ptr,
                    None => return Err(Box::new(VmError::LabelDoesNotExist {label : label.0, func : current_function.0})),
                },
            Instr::BranchOnTrue(label, f) => {
                if f(&locals)? {
                    match label_map.get(label) {
                        Some(ptr) => instr_ptr = *ptr,
                        None => return Err(Box::new(VmError::LabelDoesNotExist {label : label.0, func : current_function.0})),
                    }
                }
                else {
                    instr_ptr += 1;
                }
            },
            Instr::Return(sym) => {
                ret = Some(locals.get(sym)?);

                if stack.len() == 0 {
                    break;
                }

                Frame { instr_ptr, locals, current_function } = stack.pop().unwrap();
                // NOTE:  We don't have to check if current_function exists because if we're poping
                // then we must have called it previously.
                instrs = &func_defs_with_label.get(&current_function).unwrap().body;
                label_map = &func_defs_with_label.get(&current_function).unwrap().label_map;
            },
            Instr::LoadValue(sym, data) => {
                locals.set(sym, Data::Value(data.clone()))?;
                instr_ptr += 1;
            },
            Instr::LoadFromReturn(sym) => {
                match ret {
                    Some(ref ret) => {
                        locals.set(sym, ret.clone())?;
                        instr_ptr += 1;
                    },
                    None => return Err(Box::new(VmError::ReturnNotSet { func: current_function.0, sym: sym.0 })),
                }
            },
            Instr::Call(sym) => {
                match locals.get(sym)? {
                    Data::Func(f) => {

                        if !func_defs_with_label.contains_key(&f) {
                            return Err(Box::new(VmError::FunctionDoesNotExist(f.0)));
                        }

                        let old_function = current_function;
                        let old_instr_ptr = instr_ptr + 1;
                        let mut old_locals : Locals<T> = Locals::new(f.0);

                        current_function = f;
                        instr_ptr = 0;
                        // NOTE:  We check that the function is here earlier on in this case.
                        instrs = &func_defs_with_label.get(&current_function).unwrap().body;
                        label_map = &func_defs_with_label.get(&current_function).unwrap().label_map;
                        std::mem::swap(&mut old_locals, &mut locals);

                        stack.push(Frame { instr_ptr: old_instr_ptr
                                         , locals: old_locals
                                         , current_function: old_function
                                         });
                    },
                    _ => return Err(Box::new(VmError::AttemptToCallNonFunction { current_func: current_function.0 })),
                }

            },
            Instr::PushParam(sym) => {
                params.push(locals.get(sym)?);
                instr_ptr += 1;
            },
            Instr::PopParam(sym) => {
                match params.pop() {
                    Some(param) => locals.set(sym, param)?,
                    None => return Err(Box::new(VmError::AttemptToPopEmptyParams { current_func: current_function.0, sym: sym.0 })),
                }
                instr_ptr += 1;
            },
            Instr::LoadFromExec(sym, f) => {
                locals.set(sym, f(&locals)?)?;
                instr_ptr += 1;
            },
            Instr::LoadFunc(sym, f) => {
                locals.set(sym, Data::Func(*f))?;
                instr_ptr += 1;
            },
            Instr::SysCall(f) => {
                f(&mut locals, env)?;
                instr_ptr += 1;
            },
            Instr::LoadFromSysCall(sym, f) => {
                let result = f(&mut locals, env)?;
                locals.set(sym, result)?;
                instr_ptr += 1;
            },
        }
    }

    Ok(ret)
}

fn setup_label_map<'a, T : Clone, Env>(func_def : &'a Vec<Instr<T, Env>>, current_function : Func) -> R<FuncDefWithLabel<'a, T, Env>> {

    let mut label_map : HashMap<Label, usize> = HashMap::new();
    for (index, instr) in func_def.iter().enumerate() {
        match instr {
            Instr::Label(label) => {
                match label_map.insert( *label, index ) {
                    Some(_) => return Err(Box::new(VmError::RedefinitionOfLabel { label : label.0, func : current_function.0})),
                    None => { },
                }
            },
            _ => { },
        }
    }

    Ok(FuncDefWithLabel { body: func_def, label_map })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_immediately_return_on_empty_entry_function() -> R<()> {
        let func_defs : HashMap<Func, Vec<Instr<u8, _>>> = HashMap::from( [(Func(0), vec![])] );

        let result = run(&func_defs, &mut 0)?;

        assert!( matches!( result, None ) );
        Ok(())
    } 

    #[test]
    fn should_return_data() -> R<()> {
        let var_sym = Symbol(0);
        let func_defs : HashMap<Func, Vec<Instr<u8, _>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(var_sym, 55)
                           , Instr::Return(var_sym)
                           ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 55 );
        }
        else { 
            assert!(false);
        }

        Ok(())
    }

    #[test]
    fn should_jump_past_early_return() -> R<()> {
        let ignore = Symbol(0);
        let ret = Symbol(1);
        let func_defs : HashMap<Func, Vec<Instr<u8, _>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(ignore, 55)
                           , Instr::LoadValue(ret, 10)
                           , Instr::Jump(Label(0))
                           , Instr::Return(ignore)
                           , Instr::Label(Label(0))
                           , Instr::Return(ret)
                           ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 10 );
        }
        else { 
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_push_and_pop_of_param() -> R<()> {
        let init = Symbol(0);
        let ret = Symbol(1);
        let func_defs : HashMap<Func, Vec<Instr<u8, _>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 10)
                           , Instr::PushParam(init)
                           , Instr::PopParam(ret)
                           , Instr::Return(ret)
                           ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 10 );
        }
        else { 
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_sys_call() -> R<()> {
        let init = Symbol(0);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 10)
                           , Instr::SysCall(Box::new(
                                move |locals, env| { 
                                    if let Data::Value(x) = locals.get(&init)? {
                                       *env = x;
                                    }
                                    return Ok(()); 
                                })) 
                            ])
            ]);

        let mut env : usize = 0;
        let result = run(&func_defs, &mut env)?;

        assert!( matches!( result, None ) );
        assert_eq!( env, 10 );

        Ok(())
    } 

    #[test]
    fn should_handle_load_from_sys_call() -> R<()> {
        let init = Symbol(0);
        let ret = Symbol(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                           , Instr::LoadFromSysCall(ret, Box::new(
                                move |locals, env| { 
                                    if let Data::Value(x) = locals.get(&init)? {
                                        return Ok(Data::Value(*env + x));
                                    }
                                    panic!("!");
                                })) 
                            , Instr::Return(ret),
                            ])
            ]);

        let mut env : usize = 11;
        if let Data::Value( result ) = run(&func_defs, &mut env)?.unwrap() {
            assert_eq!( result, 18 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_load_from_exec() -> R<()> {
        let init = Symbol(0);
        let ret = Symbol(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                            , Instr::LoadFromExec(ret, Box::new(
                                move |locals| { 
                                    if let Data::Value(x) = locals.get(&init)? {
                                        return Ok(Data::Value(x + 11));
                                    }
                                    panic!("!");
                                })) 
                            , Instr::Return(ret),
                            ])
            ]);

        let mut env : usize = 11;
        if let Data::Value( result ) = run(&func_defs, &mut env)?.unwrap() {
            assert_eq!( result, 18 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_branch_on_true_when_true() -> R<()> {
        let init = Symbol(0);
        let ignore = Symbol(1);
        let label = Label(0);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                           , Instr::LoadValue(ignore, 11)
                           , Instr::BranchOnTrue(label, Box::new(
                                move |locals| { 
                                    if let Data::Value(x) = locals.get(&init)? {
                                        Ok(x == 7)
                                    }
                                    else {
                                        Ok(false)
                                    }
                                })) 
                            , Instr::Return(ignore)
                            , Instr::Label(label)  
                            , Instr::Return(init)
                            ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 7 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_branch_on_true_when_false() -> R<()> {
        let init = Symbol(0);
        let ignore = Symbol(1);
        let label = Label(0);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                           , Instr::LoadValue(ignore, 11)
                           , Instr::BranchOnTrue(label, Box::new(
                                move |locals| { 
                                    if let Data::Value(x) = locals.get(&init)? {
                                        Ok(x == 0)
                                    }
                                    else {
                                        Ok(false)
                                    }
                                })) 
                            , Instr::Return(init)
                            , Instr::Label(label)  
                            , Instr::Return(ignore)
                            ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 7 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_call() -> R<()> {
        let init = Symbol(0);
        let f = Symbol(1);
        let f1 = Func(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                           , Instr::LoadFunc(f, f1)
                           , Instr::Call(f)
                           , Instr::Return(init)
                           ])
            , (Func(1), vec![])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 7 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_call_with_return() -> R<()> {
        let init = Symbol(0);
        let f = Symbol(1);
        let f1 = Func(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadFunc(f, f1)
                           , Instr::Call(f)
                           , Instr::LoadFromReturn(init)
                           , Instr::Return(init)
                           ])
            ,(Func(1), vec![ Instr::LoadValue(init, 7)
                           , Instr::Return(init)
                           ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 7 );
        }
        else {
            assert!(false);
        }

        Ok(())
    } 

    #[test]
    fn should_handle_params_across_function_calls() -> R<()> {
        let init = Symbol(0);
        let init2 = Symbol(1);
        let f = Symbol(1);
        let f1 = Func(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(init, 7)
                           , Instr::PushParam(init)
                           , Instr::LoadValue(init, 11)
                           , Instr::PushParam(init)
                           , Instr::LoadFunc(f, f1)
                           , Instr::Call(f)
                           , Instr::LoadFromReturn(init)
                           , Instr::Return(init)
                           ])
            ,(Func(1), vec![ Instr::PopParam(init)
                           , Instr::PopParam(init2)
                           , Instr::LoadFromExec(init, Box::new(
                                move |locals| {
                                    let a = locals.get(&init)?;
                                    let b = locals.get(&init2)?;

                                    match (a, b) {
                                        (Data::Value(a), Data::Value(b)) => Ok(Data::Value(a + b)),
                                        _ => Ok(Data::Value(0)),
                                    }
                                }))
                            , Instr::Return(init)
                            ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 18 );
        }
        else {
            assert!(false);
        }

        Ok(())
    }

    #[test]
    fn should_preserve_symbol_isolation_between_calls() -> R<()> {
        let sym = Symbol(0);
        let f = Func(1);
        let f1 = Symbol(1);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(sym, 2)
                           , Instr::LoadFunc(f1, f)
                           , Instr::Call(f1)
                           , Instr::Return(sym)
                           ])
            ,(Func(1), vec![ Instr::LoadValue(sym, 7)
                           ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 2 );
        }
        else {
            assert!(false);
        }

        Ok(())
    }

    #[test]
    fn should_handle_recursive_behavior() -> R<()> {
        let input = Symbol(1);
        let ret = Symbol(2);
        let next = Symbol(3);
        let f = Func(1);
        let f1 = Symbol(0);
        let end = Label(0);
        let func_defs : HashMap<Func, Vec<Instr<usize, usize>>> = HashMap::from( 
            [(Func(0), vec![ Instr::LoadValue(input, 5)
                           , Instr::PushParam(input)
                           , Instr::LoadFunc(f1, f)
                           , Instr::Call(f1)
                           , Instr::LoadFromReturn(ret)
                           , Instr::Return(ret)
                           ])
            ,(Func(1), vec![ Instr::PopParam(input)
                           , Instr::BranchOnTrue( end, Box::new(
                                move |locals| {
                                    if let Data::Value( 0 ) = locals.get(&input)? {
                                        Ok(true)
                                    }
                                    else {
                                        Ok(false)
                                    }
                                } ) )
                            , Instr::LoadFromExec(next, Box::new(
                                move |locals| {
                                    if let Data::Value( x ) = locals.get(&input)? {
                                        Ok(Data::Value( x - 1 ))
                                    }
                                    else {
                                        Ok(Data::Value(0))
                                    }
                                }))
                            , Instr::PushParam(next)
                            , Instr::LoadFunc(f1, f)
                            , Instr::Call(f1)
                            , Instr::LoadFromReturn(ret)
                            , Instr::LoadFromExec(ret, Box::new(
                                move |locals| {
                                    let a = locals.get(&input)?;
                                    let b = locals.get(&ret)?;
                                    match (a, b) {
                                        (Data::Value(a), Data::Value(b)) => Ok(Data::Value( a + b )),
                                        _ => panic!("!"),
                                    }
                                }))
                            , Instr::Return(ret)
                            , Instr::Label(end)
                            , Instr::Return(input)
                            ])
            ]);

        if let Data::Value( result ) = run(&func_defs, &mut 0)?.unwrap() {
            assert_eq!( result, 15 );
        }
        else {
            assert!(false);
        }

        Ok(())
    }
}
