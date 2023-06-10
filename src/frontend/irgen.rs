use std::collections::HashMap;

use super::context::Type;
use super::{ast::*, context::Context};
use super::{init_padding_float, init_padding_int, ExpValue, RetInitVec};
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::frontend::typesearch::TypeProcess;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::{self, Function};
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::utility::{ObjPool, ObjPtr};

pub struct Kit<'a> {
    pub context_mut: &'a mut Context<'a>,
    pool_inst_mut: &'a mut ObjPool<Inst>,
    pool_func_mut: &'a mut ObjPool<Function>,
    pool_bb_mut: &'a mut ObjPool<BasicBlock>,
}

impl Kit<'_> {
    pub fn init_external_funcs(&mut self) {
        let inst_getint = self.pool_func_mut.new_function();
        inst_getint.as_mut().set_return_type(IrType::Int);

        let inst_getch = self.pool_func_mut.new_function();
        inst_getch.as_mut().set_return_type(IrType::Int);

        let inst_getfloat = self.pool_func_mut.new_function();
        inst_getfloat.as_mut().set_return_type(IrType::Float);

        let inst_getarray = self.pool_func_mut.new_function();
        let param_getarray = self.pool_inst_mut.make_param(IrType::IntPtr);
        inst_getarray
            .as_mut()
            .set_parameter("a".to_string(), param_getarray); //
        inst_getarray.as_mut().set_return_type(IrType::Int);

        let inst_getfarray = self.pool_func_mut.new_function();
        let param_getfarray = self.pool_inst_mut.make_param(IrType::FloatPtr);
        inst_getfarray
            .as_mut()
            .set_parameter("a".to_string(), param_getfarray); //
        inst_getfarray.as_mut().set_return_type(IrType::Int);

        let inst_putint = self.pool_func_mut.new_function();
        let param_putint = self.pool_inst_mut.make_param(IrType::Int);
        inst_putint
            .as_mut()
            .set_parameter("a".to_string(), param_putint); //
        inst_putint.as_mut().set_return_type(IrType::Void);

        let inst_putch = self.pool_func_mut.new_function();
        let param_putch = self.pool_inst_mut.make_param(IrType::Int);
        inst_putch
            .as_mut()
            .set_parameter("a".to_string(), param_putch); //
        inst_putch.as_mut().set_return_type(IrType::Void);

        let inst_putfloat = self.pool_func_mut.new_function();
        let param_putfloat = self.pool_inst_mut.make_param(IrType::Float);
        inst_putfloat
            .as_mut()
            .set_parameter("a".to_string(), param_putfloat); //
        inst_putfloat.as_mut().set_return_type(IrType::Void);

        let inst_putarray = self.pool_func_mut.new_function();
        let param_putarray1 = self.pool_inst_mut.make_param(IrType::Int);
        let param_putarray2 = self.pool_inst_mut.make_param(IrType::IntPtr);
        inst_putarray
            .as_mut()
            .set_parameter("a".to_string(), param_putarray1); //
        inst_putarray
            .as_mut()
            .set_parameter("b".to_string(), param_putarray2); //
        inst_putarray.as_mut().set_return_type(IrType::Void);

        let inst_putfarray = self.pool_func_mut.new_function();
        let param_putfarray1 = self.pool_inst_mut.make_param(IrType::Int);
        let param_putfarray2 = self.pool_inst_mut.make_param(IrType::FloatPtr);
        inst_putfarray
            .as_mut()
            .set_parameter("a".to_string(), param_putfarray1); //
        inst_putfarray
            .as_mut()
            .set_parameter("b".to_string(), param_putfarray2); //
        inst_putfarray.as_mut().set_return_type(IrType::Void);

        self.context_mut
            .module_mut
            .push_function("getin".to_string(), inst_getint);
        self.context_mut
            .module_mut
            .push_function("getch".to_string(), inst_getch);
        self.context_mut
            .module_mut
            .push_function("getfloat".to_string(), inst_getfloat);
        self.context_mut
            .module_mut
            .push_function("getarray".to_string(), inst_getarray);
        self.context_mut
            .module_mut
            .push_function("getfarray".to_string(), inst_getfarray);
        self.context_mut
            .module_mut
            .push_function("putint".to_string(), inst_putint);
        self.context_mut
            .module_mut
            .push_function("putch".to_string(), inst_putch);
        self.context_mut
            .module_mut
            .push_function("putfloat".to_string(), inst_putfloat);
        self.context_mut
            .module_mut
            .push_function("putarray".to_string(), inst_putarray);
        self.context_mut
            .module_mut
            .push_function("putfarray".to_string(), inst_putfarray);
    }

    pub fn push_inst(&mut self, inst_ptr: ObjPtr<Inst>) {
        self.context_mut.push_inst_bb(inst_ptr);
    }

    pub fn phi_padding_allfunctions(&mut self) {
        //填充所有函数中的phi
        // println!("填phi开始");
        let mut vec_funcs = self.get_functions().unwrap().clone();
        for func in vec_funcs {
            // println!(
            //     "填phi,函数头basicblock名:{:?}",
            //     func.as_ref().get_head().get_name()
            // );
            if func.is_empty_bb() {
                continue;
            }
            let head_bb_temp = func.as_ref().get_head();
            self.phi_padding_bb(head_bb_temp); //填充该函数中所有bb中的phi
        }
    }

    pub fn get_functions(&self) -> Option<Vec<(ObjPtr<Function>)>> {
        //获得函数ptr
        let mut vec_funcs = vec![];
        if !self.context_mut.module_mut.get_all_func().is_empty() {
            vec_funcs = self
                .context_mut
                .module_mut
                .get_all_func()
                .iter()
                .map(|(x, y)| *y)
                .collect();
            Some(vec_funcs)
        } else {
            None
        }
    }

    pub fn phi_padding_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        //填充该bb中的phi
        // println!("bbnow:name:{:?}", bb.get_name());
        let bbname = bb.get_name();
        let option_phi = self.context_mut.phi_list.get(bbname);
        let mut vec_phi = vec![];
        let mut is_padded = false;
        if let Some((vec_phi_temp, is_padded_temp)) = option_phi {
            // println!("有phi");
            vec_phi = vec_phi_temp.clone();
            is_padded = *is_padded_temp;
        } else {
            // println!("没phi");
        }
        if !is_padded {
            //没被填过
            // println!("phi_list长度:{:?}", vec_phi.len());
            for (name_changed, inst_phi, phi_is_padded) in vec_phi.clone() {
                // println!("填phi{:?}:{:?}", name_changed, inst_phi.get_kind());
                self.phi_padding_inst(&name_changed, inst_phi, bb);
            }
        } else {
            // println!("phi填过了,跳过");
        }
        if !is_padded {
            self.context_mut
                .phi_list
                .insert(bbname.to_string(), (vec![], true));
        }
        //判断是否是最后一个bb
        let bb_success = bb.get_next_bb();
        for bb_next in bb_success {
            if bb_next.get_name() == bb.get_name() {
                // println!("自己插过phi了");
                continue;
            }
            if let Some((vec_phi_temp, is_padded_temp)) =
                self.context_mut.phi_list.get(bb_next.get_name())
            {
                // println!("有phi");
                vec_phi = vec_phi_temp.clone();
                is_padded = *is_padded_temp;
                if is_padded {
                    continue;
                }
            } else {
                // println!("没phi");
                continue;
            }
            self.phi_padding_bb(*bb_next);
        }
    }

    pub fn phi_padding_inst(
        &mut self,
        name_changed: &str,
        inst_phi: ObjPtr<Inst>,
        bb: ObjPtr<BasicBlock>,
    ) {
        //填充bb中的变量为name_changed的inst_phi
        let vec_pre = bb.get_up_bb();
        for pre in vec_pre {
            let inst_find = self.find_var(*pre, &name_changed).unwrap();
            inst_phi.as_mut().add_operand(inst_find); //向上找,填充
                                                      // println!("其参数为:{:?}", inst_find.get_kind());
        }
    }

    pub fn find_var(
        &mut self,
        bb: ObjPtr<BasicBlock>,
        var_name_changed: &str,
    ) -> Result<ObjPtr<Inst>, Error> {
        // println!("在bb:{:?}中找", bb.get_name());
        let bbname = bb.get_name();
        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(var_name_changed));
        if let Some(inst_var) = inst_opt {
            // println!("找到了,返回{:?}", inst_var.get_kind());
            Ok(*inst_var)
        } else {
            // println!("没找到,插phi");
            let sym_opt = self.context_mut.symbol_table.get(var_name_changed);
            if let Some(sym) = sym_opt {
                // let inst_phi = self
                //     .push_phi(var_name_changed.to_string(), sym.tp, bb)
                //     .unwrap();
                match sym.tp {
                    Type::ConstFloat | Type::Float => {
                        let inst_phi = self.pool_inst_mut.make_float_phi();
                        bb.as_mut().push_front(inst_phi);
                        //填phi
                        if let Some(inst_map) = self.context_mut.bb_map.get_mut(bbname) {
                            inst_map.insert(var_name_changed.to_string(), inst_phi);
                        } else {
                            let mut map = HashMap::new();
                            map.insert(var_name_changed.to_string(), inst_phi);
                            self.context_mut.bb_map.insert(bbname.to_string(), map);
                        }
                        self.phi_padding_inst(var_name_changed, inst_phi, bb);
                        // self.context_mut
                        //     .bb_map
                        //     .get_mut(bbname)
                        //     .and_then(|var_inst_map_insert| {
                        //         var_inst_map_insert.insert(var_name_changed.to_string(), inst_phi)
                        //     });

                        Ok(inst_phi)
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_phi = self.pool_inst_mut.make_int_phi();
                        bb.as_mut().push_front(inst_phi);
                        //填phi
                        // println!("没找到,向{:?}里插phi", bb.get_name());
                        // self.context_mut.update_var_scope(
                        //     var_name_changed,
                        //     inst_phi,
                        //     bb.get_name(),
                        // );

                        if let Some(inst_map) = self.context_mut.bb_map.get_mut(bbname) {
                            inst_map.insert(var_name_changed.to_string(), inst_phi);
                        } else {
                            let mut map = HashMap::new();
                            map.insert(var_name_changed.to_string(), inst_phi);
                            self.context_mut.bb_map.insert(bbname.to_string(), map);
                        }
                        self.phi_padding_inst(var_name_changed, inst_phi, bb);
                        // self.context_mut
                        //     .bb_map
                        //     .get_mut(bbname)
                        //     .and_then(|var_inst_map_insert| {
                        //         var_inst_map_insert.insert(var_name_changed.to_string(), inst_phi)
                        //     });

                        Ok(inst_phi)
                    }
                }
            } else {
                // println!("没找到符号{:?}", var_name_changed);
                // println!("符号表长度:{:?}", self.context_mut.symbol_table.len());
                Err(Error::FindVarError)
            }
        }
    }

    pub fn add_var(
        &mut self,
        s: &str,
        tp: Type,
        is_array: bool,
        is_param: bool,
        array_inst: Option<ObjPtr<Inst>>,
        global_inst: Option<ObjPtr<Inst>>,
        dimension: Vec<i32>,
    ) {
        self.context_mut.add_var(
            s,
            tp,
            is_array,
            is_param,
            array_inst,
            global_inst,
            dimension,
        );
    }

    pub fn param_used(&mut self, s: &str) {
        // let inst = self.context_mut.module_mut.get_var(s);

        let mut name_changed = " ".to_string();
        let mut layer_var = 0;

        self.context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, layer)| {
                name_changed = last_elm.clone();
                layer_var = *layer;
                self.context_mut.symbol_table.get(last_elm)
            }); //获得改名后的名字

        self.context_mut
            .param_usage_table
            .insert(name_changed, true);
    }

    // pub fn update_var(&mut self, s: &str, inst: ObjPtr<Inst>) -> bool {
    //     self.context_mut.update_var_scope(s, inst)
    // }

    pub fn push_phi(
        &mut self,
        name: String,
        tp: Type,
        bb: ObjPtr<BasicBlock>,
    ) -> Result<ObjPtr<Inst>, Error> {
        match tp {
            Type::ConstFloat | Type::Float => {
                // println!()
                let inst_phi = self.pool_inst_mut.make_float_phi();
                // println!("指令{:?}插入bb{:?}中", inst_phi.get_kind(), bb.get_name());
                bb.as_mut().push_front(inst_phi);
                self.context_mut
                    .update_var_scope(name.as_str(), inst_phi, bb.get_name());
                if let Some((phi_list, _)) = self.context_mut.phi_list.get_mut(bb.get_name()) {
                    //如果有philist
                    // println!(
                    //     "有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     name
                    // );
                    phi_list.push((name, inst_phi, false));
                } else {
                    // println!(
                    //     "没有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     name
                    // );
                    //如果没有,生成新的philist,插入
                    let mut vec = vec![];
                    vec.push((name, inst_phi, false));
                    self.context_mut
                        .phi_list
                        .insert(bb.get_name().to_string(), (vec, false));
                }
                Ok(inst_phi)
            }
            Type::ConstInt | Type::Int => {
                let inst_phi = self.pool_inst_mut.make_int_phi();
                // println!("指令{:?}插入bb{:?}中", inst_phi.get_kind(), bb.get_name());
                bb.as_mut().push_front(inst_phi);
                self.context_mut
                    .update_var_scope(name.as_str(), inst_phi, bb.get_name());
                if let Some((phi_list, is_padded)) =
                    self.context_mut.phi_list.get_mut(bb.get_name())
                {
                    //如果有philist
                    // println!(
                    //     "有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     bb.get_name()
                    // );
                    phi_list.push((name, inst_phi, false));
                } else {
                    // println!(
                    //     "没有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     bb.get_name()
                    // );
                    //如果没有,生成新的philist,插入
                    let mut vec = vec![];
                    vec.push((name, inst_phi, false));
                    self.context_mut
                        .phi_list
                        .insert(bb.get_name().to_string(), (vec, false));
                }
                Ok(inst_phi)
            }
        }
    }

    pub fn get_var_symbol(&mut self, s: &str) -> Result<Symbol, Error> {
        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, _)| self.context_mut.symbol_table.get(last_elm))
            .map(|x| x.clone());
        if let Some(sym) = sym_opt {
            return Ok(sym);
        }
        Err(Error::VariableNotFound)
    }

    // pub fn get_var_symbol()

    pub fn get_var(
        &mut self,
        s: &str,
        offset: Option<ObjPtr<Inst>>,
        bool_get_ptr: bool,
    ) -> Result<(ObjPtr<Inst>, Symbol), Error> {
        // let bb = self.context_mut.bb_now_mut;
        match self.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb) => {
                if let Some((inst, symbol)) = self.get_var_bb(s, bb, offset, bool_get_ptr) {
                    return Ok((inst, symbol));
                }
            }
            InfuncChoice::NInFunc() => {
                let inst = self.context_mut.module_mut.get_var(s);
                let mut name_changed = " ".to_string();
                let mut layer_var = 0;

                let sym_opt = self
                    .context_mut
                    .var_map
                    .get(s)
                    .and_then(|vec_temp| vec_temp.last())
                    .and_then(|(last_elm, layer)| {
                        name_changed = last_elm.clone();
                        layer_var = *layer;
                        self.context_mut.symbol_table.get(last_elm)
                    })
                    .map(|x| x.clone());
                let mut bbname = "notinblock";

                let inst_opt = self
                    .context_mut
                    .bb_map
                    .get(bbname)
                    .and_then(|var_inst_map| var_inst_map.get(&name_changed));

                if let Some(sym) = sym_opt {
                    // println!("找到变量{:?}",s);
                    return Ok((inst, sym));
                }
                // InfuncChoice::NInFunc() => {
                //     return todo!();;
                // }
            }
        }
        // println!("没找到变量:{:?}",s);
        return Err(Error::VariableNotFound);
    }

    pub fn get_var_bb(
        &mut self,
        s: &str,
        bb: ObjPtr<BasicBlock>,
        offset: Option<ObjPtr<Inst>>,
        bool_get_ptr: bool,
    ) -> Option<(ObjPtr<Inst>, Symbol)> {
        let mut name_changed = " ".to_string();
        let mut layer_var = 0;

        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, layer)| {
                name_changed = last_elm.clone();
                layer_var = *layer;
                self.context_mut.symbol_table.get(last_elm)
            })
            .map(|x| x.clone());

        let mut bbname = bb.as_ref().get_name();

        // let mut is_const = false;

        if layer_var == -1 {
            bbname = "notinblock"; //全局变量,const和一般类型需要分开处理吗?
        }
        // else if layer_var == 0 {
        //     // bbname = "params";
        //     if let Some(is_used) = self.context_mut.param_usage_table.get(&name_changed) {
        //         if !is_used {
        //             // println!("进来了");
        //             bbname = "params";
        //         }

        //     }
        // }//函数参数不再单独处理

        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(&name_changed));

        if let Some(sym) = sym_opt {
            // println!("进来了");
            // println!("找到变量{:?}",s);

            //应该先判断是不是数组，以防bbmap中找不到报错
            if let Some(inst_array) = sym.array_inst {
                // println!("找到数组变量{:?},不插phi", s);
                // println!("bbname:{:?}", bbname);
                //如果是数组
                let mut inst_ret = self.pool_inst_mut.make_int_const(-1129);
                match sym.tp {
                    Type::Float | Type::ConstFloat => {
                        //判断类型
                        if layer_var < 0 {
                            //是否是全局
                            if let Some(offset) = offset {
                                //有偏移
                                // let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                // inst_ret = self.pool_inst_mut.make_global_float_array_load(ptr);
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                                                                               // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                               // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                               // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                                                              // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                        // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                        // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                //没给偏移
                                if bool_get_ptr {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(ptr_array, inst_offset_temp); //获得特定指针
                                                                                                  // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                  // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                  // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                            //  else {
                            //     //没偏移
                            //     let ptr =
                            //         self.pool_inst_mut.make_global_float_array_load(inst_array);
                            //     inst_ret = self.pool_inst_mut.make_gep(ptr, );
                            //     self.context_mut.push_inst_bb(inst_ret);
                            // }
                        } else {
                            //不是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    inst_ret = self.pool_inst_mut.make_gep(inst_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr);
                                    // inst_ret = self.pool_inst_mut.make_gep(inst_array, offset);
                                    // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                                                                                   // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                   // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                   // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                                                                     // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                                //没偏移
                            }
                        }
                    }
                    Type::Int | Type::ConstInt => {
                        if layer_var < 0 {
                            //是否是全局
                            if let Some(offset) = offset {
                                //有偏移
                                // let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                // inst_ret = self.pool_inst_mut.make_global_float_array_load(ptr);
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                                                                               // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                               // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                               // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                                                              // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    inst_ret = self.pool_inst_mut.make_int_load(ptr); //获得元素值
                                                                                      // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                      // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                //没给偏移
                                if bool_get_ptr {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(ptr_array, inst_offset_temp); //获得特定指针
                                                                                                  // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                  // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                  // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                            //  else {
                            //     //没偏移
                            //     let ptr =
                            //         self.pool_inst_mut.make_global_float_array_load(inst_array);
                            //     inst_ret = self.pool_inst_mut.make_gep(ptr, );
                            //     self.context_mut.push_inst_bb(inst_ret);
                            // }
                        } else {
                            //不是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    inst_ret = self.pool_inst_mut.make_gep(inst_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                    inst_ret = self.pool_inst_mut.make_int_load(ptr);
                                    // inst_ret = self.pool_inst_mut.make_gep(inst_array, offset);
                                    // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                                                                                   // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                   // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                   // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                                                                     // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                                //没偏移
                            }
                        }
                    }
                }
                return Some((inst_ret, sym));
            } else {
                //不是数组的情况
                if layer_var < 0 {
                    // println!("找到全局变量{:?},不插phi", s);
                    // println!("bbname:{:?}", bbname);
                    //全局也肯定能找到
                    //如果是全局变量
                    // let mut inst_ret = self.pool_inst_mut.make_int_const(-1129);
                    // bbname = "notinblock";//全局变量,const和一般类型需要分开处理吗?
                    //这里返回一条load指令
                    match sym.tp {
                        Type::ConstFloat | Type::Float => {
                            if let Some(inst_global) = sym.global_inst {
                                //全局也肯定能找到
                                let inst_ret =
                                    self.pool_inst_mut.make_global_float_load(inst_global);
                                self.context_mut.push_inst_bb(inst_ret); //这里
                                return Some((inst_ret, sym));
                            }

                            // return Some((inst_load, sym));
                        }
                        Type::ConstInt | Type::Int => {
                            if let Some(inst_global) = sym.global_inst {
                                let inst_ret = self.pool_inst_mut.make_global_int_load(inst_global);
                                self.context_mut.push_inst_bb(inst_ret); //这里
                                return Some((inst_ret, sym));
                            }
                            // return Some((inst_load, sym));
                        }
                    }
                } else {
                    if let Some(inst) = inst_opt {
                        // println!("找到变量{:?},不插phi", s);
                        // println!("bbname:{:?}", bbname);
                        //找到变量
                        let mut inst_ret = *inst;
                        if layer_var < 0 {
                            //如果是全局变量
                            // bbname = "notinblock";//全局变量,const和一般类型需要分开处理吗?
                            //这里返回一条load指令
                            match sym.tp {
                                Type::ConstFloat | Type::Float => {
                                    inst_ret = self.pool_inst_mut.make_global_float_load(inst_ret);
                                    self.context_mut.push_inst_bb(inst_ret); //这里
                                                                             // return Some((inst_load, sym));
                                }
                                Type::ConstInt | Type::Int => {
                                    inst_ret = self.pool_inst_mut.make_global_int_load(inst_ret);
                                    self.context_mut.push_inst_bb(inst_ret);
                                    // return Some((inst_load, sym));
                                }
                            }
                        }

                        return Some((inst_ret, sym));
                    } else {
                        // println!("没找到变量{:?},插phi", s);
                        // println!("bbname:{:?}", bbname);

                        //没找到
                        // bb.as_ref().
                        match sym.tp {
                            Type::ConstFloat | Type::Float => {
                                let phi_inst = self
                                    .push_phi(name_changed.clone(), Type::Float, bb)
                                    .unwrap();
                                // if let Some(vec) = self.context_mut.phi_list.get_mut(bbname) {
                                //     //有philist,直接加入philist中
                                //     vec.push((name_changed.clone(), phi_inst));
                                // } else {
                                //     //该bb没有philist,新建philist,加入philist中
                                //     let mut v = vec![];
                                //     v.push((name_changed.clone(), phi_inst));
                                //     self.context_mut.phi_list.insert(bbname.to_string(), v);
                                // }
                                return Some((phi_inst, sym));
                            }
                            Type::ConstInt | Type::Int => {
                                let phi_inst =
                                    self.push_phi(name_changed.clone(), Type::Int, bb).unwrap();
                                // if let Some(vec) = self.context_mut.phi_list.get_mut(bbname) {
                                //     //有philist,直接加入philist中
                                //     vec.push((name_changed.clone(), phi_inst));
                                // } else {
                                //     //该bb没有philist,新建philist,加入philist中
                                //     let mut v = vec![];
                                //     v.push((name_changed.clone(), phi_inst));
                                //     self.context_mut.phi_list.insert(bbname.to_string(), v);
                                // }
                                return Some((phi_inst, sym));
                            }
                        }

                        // let phiinst_mut = phiinst.as_mut();
                        // let bb_mut = bb.as_mut();
                        // for preccessor in bb_mut.get_up_bb() {
                        //     if let Some((temp, symbol)) = self.get_var_bb(s, *preccessor) {
                        //         phiinst_mut.add_operand(temp);
                        //     }
                        // }
                        // return Option::Some((phiinst, sym));
                    }
                }
            }
        }
        Option::None
    }
}

pub fn irgen(
    compunit: &mut CompUnit,
    module_mut: &mut Module,
    pool_inst_mut: &mut ObjPool<Inst>,
    pool_bb_mut: &mut ObjPool<BasicBlock>,
    pool_func_mut: &mut ObjPool<Function>,
) {
    let mut pool_scope = ObjPool::new();
    let context_mut = pool_scope.put(Context::make_context(module_mut)).as_mut();
    let mut kit_mut = Kit {
        context_mut,
        pool_inst_mut,
        pool_bb_mut,
        pool_func_mut,
    };
    kit_mut.init_external_funcs();
    compunit.process(1, &mut kit_mut);
    kit_mut.phi_padding_allfunctions();
}

#[derive(Clone, Copy)]
pub enum InfuncChoice {
    InFunc(ObjPtr<BasicBlock>),
    NInFunc(),
}

pub trait Process {
    type Ret;
    type Message;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error>;
}

impl Process for CompUnit {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        for item in &mut self.global_items {
            item.process(1, kit_mut);
        }
        // println!(
        //     "结束，符号表长度为:{:?}",
        //     kit_mut.context_mut.symbol_table.len()
        // );
        // for i in &kit_mut.context_mut.symbol_table {
        //     println!("有变量:{:?}", i.0);
        // }

        // kit_mut.phi_padding_allfunctions();
        return Ok(1);
        todo!();
    }
}

impl Process for GlobalItems {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::Decl(decl) => {
                decl.process(1, kit_mut).unwrap();
                Ok(1)
            }
            Self::FuncDef(funcdef) => {
                funcdef.process(true, kit_mut);
                Ok(1)
            }
        }
        // todo!();
    }
}

impl Process for Decl {
    type Ret = i32;
    type Message = (i32);

    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::ConstDecl(constdecl) => {
                constdecl.process(input, kit_mut).unwrap();
                return Ok(1);
            }
            Self::VarDecl(vardef) => {
                vardef.process(input, kit_mut).unwrap();
                return Ok(1);
            }
        }
        todo!();
    }
}

impl Process for ConstDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.const_def_vec {
                    if def.const_exp_vec.is_empty() {
                        //非数组
                        let (mut inst_ptr, mut val, _) = def
                            .const_init_val
                            .process((Type::ConstInt, vec![]), kit_mut)
                            .unwrap();

                        let mut bond = 0;
                        match val {
                            //构造const指令
                            ExpValue::Int(i) => {
                                bond = i;
                                if kit_mut.context_mut.get_layer() < 0 {
                                    inst_ptr = kit_mut.pool_inst_mut.make_global_int_const(bond);
                                } else {
                                    inst_ptr = kit_mut.pool_inst_mut.make_int_const(i);
                                    kit_mut.context_mut.push_inst_bb(inst_ptr); //update会将全局变量放入module中不会将局部变量放入bb中
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                        if kit_mut.context_mut.get_layer() < 0 {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstInt,
                                false,
                                false,
                                None,
                                Some(inst_ptr),
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        } else {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstInt,
                                false,
                                false,
                                None,
                                None,
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        }

                        // inst_ptr = kit_mut.pool_inst_mut.make_global_int_const(bond);
                        //这里

                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr); //update会将全局变量放入module中不会将局部变量放入bb中
                    } else {
                        //数组
                        // let (mut inst_ptr, mut val) =
                        //     def.const_init_val.process(Type::ConstInt, kit_mut).unwrap();//获得初始值
                        let mut dimension = vec![];
                        for exp in &mut def.const_exp_vec {
                            dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                        let dimension_vec_in: Vec<_> = dimension_vec
                            .iter()
                            .map(|x| match x {
                                ExpValue::Int(i) => *i,
                                ExpValue::Float(f) => {
                                    unreachable!()
                                }
                                ExpValue::None => {
                                    unreachable!()
                                }
                                _ => {
                                    unreachable!()
                                }
                            })
                            .collect(); //生成维度vec

                        let mut length = 1;
                        for dm in &dimension_vec_in {
                            length = length * dm;
                        }

                        let (mut inst_ptr, mut val, init_vec) = def
                            .const_init_val
                            .process((Type::ConstInt, dimension_vec_in.clone()), kit_mut)
                            .unwrap(); //获得初始值
                        match init_vec {
                            RetInitVec::Float(fvec) => {
                                unreachable!()
                            }
                            RetInitVec::Int(ivec) => {
                                // println!("初始值:");
                                // for i in &ivec {
                                //     println!("{:?}", i);
                                // }
                                let inst = kit_mut.pool_inst_mut.make_int_array(length, ivec);
                                if !kit_mut.context_mut.add_var(
                                    &def.ident,
                                    Type::ConstInt,
                                    true,
                                    false,
                                    Some(inst),
                                    None,
                                    dimension_vec_in.clone(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                } //添加该变量，但没有生成实际的指令
                                kit_mut.context_mut.update_var_scope_now(&def.ident, inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                        }
                    }
                }
                return Ok(1);
            }
            BType::Float => {
                for def in &mut self.const_def_vec {
                    if def.const_exp_vec.is_empty() {
                        let (mut inst_ptr, mut val, _) = def
                            .const_init_val
                            .process((Type::ConstFloat, vec![]), kit_mut)
                            .unwrap();

                        let mut bond = 0.0;
                        match val {
                            ExpValue::Float(i) => {
                                bond = i;
                                if kit_mut.context_mut.get_layer() < 0 {
                                    inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                                } else {
                                    inst_ptr = kit_mut.pool_inst_mut.make_float_const(i);
                                    kit_mut.context_mut.push_inst_bb(inst_ptr);
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }

                        if kit_mut.context_mut.get_layer() < 0 {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstFloat,
                                false,
                                false,
                                None,
                                Some(inst_ptr),
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        } else {
                            if !kit_mut.context_mut.add_var(
                                &def.ident,
                                Type::ConstFloat,
                                false,
                                false,
                                None,
                                None,
                                Vec::new(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            }
                        }

                        // inst_ptr = kit_mut.pool_inst_mut.make_global_float_const(bond);
                        //这里
                        kit_mut
                            .context_mut
                            .update_var_scope_now(&def.ident, inst_ptr);
                    } else {
                        //数组
                        // let (mut inst_ptr, mut val) =
                        //     def.const_init_val.process(Type::ConstInt, kit_mut).unwrap();//获得初始值
                        let mut dimension = vec![];
                        for exp in &mut def.const_exp_vec {
                            dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                        let dimension_vec_in: Vec<_> = dimension_vec
                            .iter()
                            .map(|x| match x {
                                ExpValue::Int(i) => *i,
                                ExpValue::Float(f) => {
                                    unreachable!()
                                }
                                ExpValue::None => {
                                    unreachable!()
                                }
                                _ => {
                                    unreachable!()
                                }
                            })
                            .collect(); //生成维度vec

                        let mut length = 1;
                        for dm in &dimension_vec_in {
                            length = length * dm;
                        }

                        let (mut inst_ptr, mut val, init_vec) = def
                            .const_init_val
                            .process((Type::ConstFloat, dimension_vec_in.clone()), kit_mut)
                            .unwrap(); //获得初始值
                        match init_vec {
                            RetInitVec::Float(fvec) => {
                                let inst = kit_mut.pool_inst_mut.make_float_array(length, fvec);
                                if !kit_mut.context_mut.add_var(
                                    &def.ident,
                                    Type::ConstInt,
                                    true,
                                    false,
                                    Some(inst),
                                    None,
                                    dimension_vec_in.clone(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                } //添加该变量，但没有生成实际的指令
                                kit_mut.context_mut.update_var_scope_now(&def.ident, inst);
                                kit_mut.context_mut.push_inst_bb(inst);
                            }
                            RetInitVec::Int(ivec) => {
                                unreachable!()
                            }
                        }
                    }
                }
                return Ok(1);
            }
        }
        Ok(1)
    }
}

// impl Process for BType {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }

impl Process for ConstInitVal {
    type Ret = (ObjPtr<Inst>, ExpValue, RetInitVec);
    type Message = (Type, Vec<i32>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            ConstInitVal::ConstExp(constexp) => {
                let (inst, value) = constexp.process(input.0, kit_mut).unwrap();
                match value {
                    ExpValue::Float(f) => {
                        let mut vec_ret = Vec::new();
                        vec_ret.push(f);
                        Ok((inst, value, RetInitVec::Float(vec_ret)))
                    }
                    ExpValue::Int(i) => {
                        let mut vec_ret = Vec::new();
                        vec_ret.push(i);
                        Ok((inst, value, RetInitVec::Int(vec_ret)))
                    }
                    _ => {
                        unreachable!()
                    }
                }
                // Ok((inst,value,RetInitVec::Int(Vec::new())))
            }
            ConstInitVal::ConstInitValVec(constvalvec) => {
                let dimension_vec = input.1;
                let tp = input.0;
                let mut vec_ret_float = vec![];
                let mut vec_ret_int = vec![];
                let mut dimension_next = vec![];
                let mut index = 0;
                for i in &dimension_vec {
                    //构造下一级的维度vec
                    if index == 0 {
                        index = index + 1;
                        continue;
                    }
                    index = index + 1;
                    dimension_next.push(*i);
                }
                // println!("dimension:{:?}",dimension_vec.len());
                // println!("dimension_next:{:?}",dimension_next.len());

                for val in constvalvec {
                    let (_, _, vec_temp) =
                        val.process((tp, dimension_next.clone()), kit_mut).unwrap(); //子init_vec生成vec
                    match vec_temp {
                        //将子vec中值放到当前vec中
                        RetInitVec::Float(vec_float) => match tp {
                            Type::Float | Type::ConstFloat => {
                                for val_son in vec_float {
                                    vec_ret_float.push(val_son);
                                }
                            }
                            _ => {
                                unreachable!();
                            }
                        },
                        RetInitVec::Int(vec_int) => match tp {
                            Type::Int | Type::ConstInt => {
                                for val_son in vec_int {
                                    vec_ret_int.push(val_son);
                                }
                            }
                            _ => {
                                unreachable!();
                            }
                        },
                    }
                }

                match tp {
                    Type::Float | Type::ConstFloat => {
                        init_padding_float(&mut vec_ret_float, dimension_vec.clone());
                        return Ok((
                            kit_mut.pool_inst_mut.make_int_const(-1),
                            ExpValue::None,
                            RetInitVec::Float(vec_ret_float),
                        ));
                    }
                    Type::Int | Type::ConstInt => {
                        init_padding_int(&mut vec_ret_int, dimension_vec.clone());
                        return Ok((
                            kit_mut.pool_inst_mut.make_int_const(-1),
                            ExpValue::None,
                            RetInitVec::Int(vec_ret_int),
                        ));
                    }
                }

                // match tp {
                //     Type::ConstFloat |Type::Float =>{

                //     }
                //     Type::ConstInt |Type::Int =>{

                //     }
                // }
            }
        }
    }
}

impl Process for VarDecl {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self.btype {
            BType::Int => {
                for def in &mut self.var_def_vec {
                    match def {
                        VarDef::NonArrayInit((id, val)) => match val {
                            InitVal::Exp(exp) => {
                                let (mut inst_ptr, mut val) =
                                    exp.process(Type::Int, kit_mut).unwrap();

                                if kit_mut.context_mut.get_layer() < 0 {
                                    //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Int(i) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_int(i);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Int,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                            //这里
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                } else {
                                    if !kit_mut.context_mut.add_var(
                                        id,
                                        Type::Int,
                                        false,
                                        false,
                                        None,
                                        None,
                                        Vec::new(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    }
                                }
                                // println!(
                                //     "插入新变量,符号表长度为:{:?}",
                                //     kit_mut.context_mut.symbol_table.len()
                                // );
                                // for i in &kit_mut.context_mut.symbol_table {
                                //     println!("有变量:{:?}", i.0);
                                // }

                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray(id) => {
                            if kit_mut.context_mut.get_layer() == -1 {
                                //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_int(0);
                                if !kit_mut.context_mut.add_var(
                                    id,
                                    Type::Int,
                                    false,
                                    false,
                                    None,
                                    Some(inst_ptr),
                                    Vec::new(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            } else {
                                if !kit_mut.context_mut.add_var(
                                    id,
                                    Type::Int,
                                    false,
                                    false,
                                    None,
                                    None,
                                    Vec::new(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                // println!(
                                //     "插入新变量,符号表长度为:{:?}",
                                //     kit_mut.context_mut.symbol_table.len()
                                // );
                            }
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(f) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }
                            // let length_inst = kit_mut.pool_inst_mut.make_int_const(length);
                            // kit_mut.context_mut.push_inst_bb(length_inst); //这里

                            // if !kit_mut.context_mut.add_var(
                            //     &id,
                            //     Type::Int,
                            //     true,
                            //     false,
                            //     dimension_vec_in.clone(),
                            // ) {
                            //     return Err(Error::MultipleDeclaration);
                            // } //添加该变量，但没有生成实际的指令

                            // unreachable!()
                            let (mut init_vec, mut inst_vec) = val
                                .process((Type::Int, dimension_vec_in.clone(), 0, 1), kit_mut)
                                .unwrap(); //获得初始值
                                           // let inst =
                            match init_vec {
                                RetInitVec::Int(ivec) => {
                                    // println!("初始值:");
                                    // for i in &ivec {
                                    //     println!("{:?}", i);
                                    // }
                                    let inst = kit_mut.pool_inst_mut.make_int_array(length, ivec);
                                    if !kit_mut.context_mut.add_var(
                                        &id,
                                        Type::Int,
                                        true,
                                        false,
                                        Some(inst),
                                        None, //可能得改,数组的就没有对全局区域留inst
                                        dimension_vec_in.clone(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    } //添加该变量，但没有生成实际的指令
                                    kit_mut.context_mut.update_var_scope_now(&id, inst);
                                    kit_mut.context_mut.push_inst_bb(inst);
                                    // println!("没进来");
                                    for option_exp in inst_vec {
                                        // println!("进来了");
                                        if let Some((inst_val, offset_val)) = option_exp {
                                            let offset =
                                                kit_mut.pool_inst_mut.make_int_const(offset_val);
                                            let ptr = kit_mut.pool_inst_mut.make_gep(inst, offset);
                                            let inst_store =
                                                kit_mut.pool_inst_mut.make_int_store(ptr, inst_val);
                                            kit_mut.context_mut.push_inst_bb(offset);
                                            kit_mut.context_mut.push_inst_bb(ptr);
                                            kit_mut.context_mut.push_inst_bb(inst_store);
                                        }
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        VarDef::Array((id, exp_vec)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(f) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }

                            // let (mut init_vec, mut inst_vec) = val
                            //     .process((Type::Int, dimension_vec_in.clone(), 0, 1), kit_mut)
                            //     .unwrap(); //获得初始值
                            // let inst =
                            let mut ivec = vec![];
                            init_padding_int(&mut ivec, dimension_vec_in.clone());
                            let inst = kit_mut.pool_inst_mut.make_int_array(length, ivec);
                            if !kit_mut.context_mut.add_var(
                                &id,
                                Type::Int,
                                true,
                                false,
                                Some(inst),
                                None,
                                dimension_vec_in.clone(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            } //添加该变量，但没有生成实际的指令
                            kit_mut.context_mut.update_var_scope_now(&id, inst);
                            kit_mut.context_mut.push_inst_bb(inst);
                        }
                    }
                }
                Ok(1)
            }
            BType::Float => {
                for def in &mut self.var_def_vec {
                    match def {
                        VarDef::NonArrayInit((id, val)) => match val {
                            InitVal::Exp(exp) => {
                                let (mut inst_ptr, val) =
                                    exp.process(Type::Float, kit_mut).unwrap();

                                if kit_mut.context_mut.get_layer() < 0 {
                                    //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                    match val {
                                        ExpValue::Float(f) => {
                                            inst_ptr = kit_mut.pool_inst_mut.make_global_float(f);
                                            if !kit_mut.context_mut.add_var(
                                                id,
                                                Type::Float,
                                                false,
                                                false,
                                                None,
                                                Some(inst_ptr),
                                                Vec::new(),
                                            ) {
                                                return Err(Error::MultipleDeclaration);
                                            }
                                            //这里
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                                if !kit_mut.context_mut.add_var(
                                    id,
                                    Type::Float,
                                    false,
                                    false,
                                    None,
                                    None,
                                    Vec::new(),
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            }
                            InitVal::InitValVec(val_vec) => {
                                todo!()
                            }
                        },
                        VarDef::NonArray((id)) => {
                            if kit_mut.context_mut.get_layer() == -1 {
                                //设计相关(全局变量指令与局部变量不同)，全局变量得在这额外判断，放到module里
                                let inst_ptr = kit_mut.pool_inst_mut.make_global_float(0.0);
                                if !kit_mut.context_mut.add_var(
                                    id.as_str(),
                                    Type::Float,
                                    false,
                                    false,
                                    None,
                                    Some(inst_ptr),
                                    vec![],
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                                kit_mut.context_mut.update_var_scope_now(id, inst_ptr);
                            } else {
                                if !kit_mut.context_mut.add_var(
                                    id.as_str(),
                                    Type::Float,
                                    false,
                                    false,
                                    None,
                                    None,
                                    vec![],
                                ) {
                                    return Err(Error::MultipleDeclaration);
                                }
                            }
                        }
                        VarDef::ArrayInit((id, exp_vec, val)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(f) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }

                            // unreachable!()
                            let (mut init_vec, mut inst_vec) = val
                                .process((Type::Float, dimension_vec_in.clone(), 0, 1), kit_mut)
                                .unwrap(); //获得初始值
                                           // let inst =
                            match init_vec {
                                RetInitVec::Float(fvec) => {
                                    // println!("初始值:");
                                    // for i in &fvec {
                                    //     println!("{:?}", i);
                                    // }
                                    let inst = kit_mut.pool_inst_mut.make_float_array(length, fvec);
                                    if !kit_mut.context_mut.add_var(
                                        &id,
                                        Type::Float,
                                        true,
                                        false,
                                        Some(inst),
                                        None,
                                        dimension_vec_in.clone(),
                                    ) {
                                        return Err(Error::MultipleDeclaration);
                                    } //添加该变量，但没有生成实际的指令
                                    kit_mut.context_mut.update_var_scope_now(&id, inst);
                                    kit_mut.context_mut.push_inst_bb(inst);
                                    // println!("没进来");
                                    for option_exp in inst_vec {
                                        // println!("进来了");
                                        if let Some((inst_val, offset_val)) = option_exp {
                                            let offset =
                                                kit_mut.pool_inst_mut.make_int_const(offset_val);
                                            let ptr = kit_mut.pool_inst_mut.make_gep(inst, offset);
                                            let inst_store = kit_mut
                                                .pool_inst_mut
                                                .make_float_store(ptr, inst_val);
                                            kit_mut.context_mut.push_inst_bb(offset);
                                            kit_mut.context_mut.push_inst_bb(ptr);
                                            kit_mut.context_mut.push_inst_bb(inst_store);
                                        }
                                    }
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        VarDef::Array((id, exp_vec)) => {
                            let mut dimension = vec![];
                            for exp in exp_vec {
                                dimension.push(exp.process(Type::Int, kit_mut).unwrap());
                            }
                            let dimension_vec: Vec<_> = dimension.iter().map(|(_, x)| x).collect();
                            let dimension_vec_in: Vec<_> = dimension_vec
                                .iter()
                                .map(|x| match x {
                                    ExpValue::Int(i) => *i,
                                    ExpValue::Float(f) => {
                                        unreachable!()
                                    }
                                    ExpValue::None => {
                                        unreachable!()
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                })
                                .collect(); //生成维度vec

                            let mut length = 1;
                            for dm in &dimension_vec_in {
                                length = length * dm;
                            }

                            // let (mut init_vec, mut inst_vec) = val
                            //     .process((Type::Int, dimension_vec_in.clone(), 0, 1), kit_mut)
                            //     .unwrap(); //获得初始值
                            // let inst =
                            let mut fvec = vec![];
                            init_padding_float(&mut fvec, dimension_vec_in.clone());
                            let inst = kit_mut.pool_inst_mut.make_float_array(length, fvec);
                            if !kit_mut.context_mut.add_var(
                                &id,
                                Type::Float,
                                true,
                                false,
                                Some(inst),
                                None,
                                dimension_vec_in.clone(),
                            ) {
                                return Err(Error::MultipleDeclaration);
                            } //添加该变量，但没有生成实际的指令
                            kit_mut.context_mut.update_var_scope_now(&id, inst);
                            kit_mut.context_mut.push_inst_bb(inst);
                        }
                        _ => todo!(),
                    }
                }
                Ok(1)
            }
        }
    }
}
impl Process for VarDef {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for InitVal {
    type Ret = (RetInitVec, Vec<Option<(ObjPtr<Inst>, i32)>>);
    type Message = (Type, Vec<i32>, i32, usize);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            InitVal::Exp(exp) => {
                let (tp, dimension, num_precessor, layer_now) = input;
                let (inst, val) = exp.process(tp, kit_mut).unwrap();
                let mut vecf = vec![];
                let mut veci = vec![];
                let mut inst_vec = vec![];
                match val {
                    ExpValue::Float(f) => match tp {
                        Type::Float | Type::ConstFloat => {
                            vecf.push(f);
                            inst_vec.push(None);
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    ExpValue::Int(i) => match tp {
                        Type::Int | Type::ConstInt => {
                            veci.push(i);
                            inst_vec.push(None);
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    ExpValue::None => match tp {
                        Type::Float | Type::ConstFloat => {
                            vecf.push(0.0);
                            // let offset =
                            inst_vec.push(Some((inst, num_precessor)));
                        }
                        Type::Int | Type::ConstInt => {
                            veci.push(0);
                            // let offset =
                            inst_vec.push(Some((inst, num_precessor)));
                        }
                        _ => {
                            unreachable!()
                        }
                    },
                    _ => {
                        unreachable!()
                    }
                }
                match tp {
                    Type::Float | Type::ConstFloat => Ok((RetInitVec::Float(vecf), inst_vec)),
                    Type::Int | Type::ConstInt => Ok((RetInitVec::Int(veci), inst_vec)),
                }
                // Err(Error::Todo)
            }

            InitVal::InitValVec(initvec) => {
                let (tp, dimension, num_precessor, layer_now) = input;
                let mut vec_val_f = vec![];
                let mut vec_val_i = vec![];
                let mut vec_inst_init = vec![];
                let mut after = 1;
                for i in layer_now..dimension.len() {
                    after = after * dimension[i];
                } //计算当前维度每增1对应多少元素
                let mut vec_dimension_now = vec![];
                for i in (layer_now - 1)..dimension.len() {
                    vec_dimension_now.push(dimension[i]);
                } //计算当前维度每增1对应多少元素

                let mut index = 0; //当前相对位移
                for init in initvec {
                    match init {
                        InitVal::Exp(exp) => {
                            let (vec_val_temp, vec_inst_temp) = init
                                .process(
                                    (tp, dimension.clone(), num_precessor + index, layer_now),
                                    kit_mut,
                                )
                                .unwrap();
                            match vec_val_temp {
                                RetInitVec::Float(vec_f) => {
                                    for val in vec_f {
                                        vec_val_f.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(inst_list) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                                RetInitVec::Int(vec_i) => {
                                    for val in vec_i {
                                        vec_val_i.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(inst_list) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                            }

                            index = index + 1; //init为exp，相对偏移加1
                        }
                        InitVal::InitValVec(initvec) => {
                            let (vec_val_temp, vec_inst_temp) = init
                                .process(
                                    (tp, dimension.clone(), num_precessor + index, layer_now + 1),
                                    kit_mut,
                                )
                                .unwrap();
                            match vec_val_temp {
                                RetInitVec::Float(vec_f) => {
                                    for val in vec_f {
                                        vec_val_f.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(inst_list) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                                RetInitVec::Int(vec_i) => {
                                    for val in vec_i {
                                        vec_val_i.push(val);
                                    }
                                    for inst in vec_inst_temp {
                                        if let Some(inst_list) = inst {
                                            vec_inst_init.push(inst);
                                        }
                                    }
                                }
                            }
                            index = index + after; //init为vec,相对偏移加after
                        }
                    }
                }

                match tp {
                    Type::Float | Type::ConstFloat => {
                        init_padding_float(&mut vec_val_f, vec_dimension_now);
                        Ok((RetInitVec::Float(vec_val_f), vec_inst_init))
                    }
                    Type::Int | Type::ConstInt => {
                        init_padding_int(&mut vec_val_i, vec_dimension_now);
                        Ok((RetInitVec::Int(vec_val_i), vec_inst_init))
                    }
                }

                // Err(Error::Todo)
            }
        }
    }
}
impl Process for FuncDef {
    type Ret = i32;
    type Message = bool;
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Self::NonParameterFuncDef((tp, id, blk)) => {
                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => func_mut.set_return_type(IrType::Void),
                    FuncType::Int => func_mut.set_return_type(IrType::Int),
                    FuncType::Float => func_mut.set_return_type(IrType::Float),
                }
                kit_mut.context_mut.bb_now_set(bb);
                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                blk.process((None, None), kit_mut);
                kit_mut.context_mut.delete_layer();
                return Ok(1);
            }
            Self::ParameterFuncDef((tp, id, params, blk)) => {
                kit_mut.context_mut.add_layer();
                let func_ptr = kit_mut.pool_func_mut.new_function();
                let func_mut = func_ptr.as_mut();
                let bb = kit_mut.pool_bb_mut.new_basic_block(id.clone());
                func_mut.insert_first_bb(bb);
                match tp {
                    FuncType::Void => func_mut.set_return_type(IrType::Void),
                    FuncType::Int => func_mut.set_return_type(IrType::Int),
                    FuncType::Float => func_mut.set_return_type(IrType::Float),
                }

                kit_mut
                    .context_mut
                    .push_func_module(id.to_string(), func_ptr);
                let params_vec = params.process(1, kit_mut).unwrap();

                kit_mut.context_mut.bb_now_set(bb); //和for语句顺序改过

                for (name, param) in params_vec {
                    // kit_mut.add_var(&name, tp, is_array, dimension)
                    kit_mut.context_mut.update_var_scope_now(&name, param); //新增
                    func_mut.set_parameter(name, param); //这里
                }

                blk.process((None, None), kit_mut);
                kit_mut.context_mut.delete_layer();
                return Ok(1);
            }
        }
        // module.push_function(name, function);
        todo!();
    }
}

// impl Process for FuncType {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }
impl Process for FuncFParams {
    type Ret = Vec<(String, ObjPtr<Inst>)>;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let mut vec = vec![];
        for param in &mut self.func_fparams_vec {
            let p = param.process(input, kit_mut).unwrap();
            vec.push(p);
        }
        Ok(vec)
    }
}

impl Process for FuncFParam {
    type Ret = (String, ObjPtr<Inst>);
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            FuncFParam::Array((tp, id, vec)) => {
                //vec中存储的是从第二维开始的维度信息，第一维默认存在且为空
                match tp {
                    BType::Int => {
                        let param = kit_mut.pool_inst_mut.make_param(IrType::IntPtr);
                        let mut dimension_vec = vec![];
                        let mut dimension_vec_in = vec![];
                        for exp in vec {
                            dimension_vec.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        dimension_vec_in.push(-1);
                        for (inst, dimension) in dimension_vec {
                            match dimension {
                                ExpValue::Int(i) => {
                                    dimension_vec_in.push(i);
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        kit_mut.context_mut.add_var(
                            id,
                            Type::Int,
                            true,
                            true,
                            Some(param),
                            None,
                            dimension_vec_in,
                        );
                        //这里
                        kit_mut.context_mut.update_var_scope_now(id, param);
                        Ok((id.clone(), param))
                    }
                    BType::Float => {
                        let param = kit_mut.pool_inst_mut.make_param(IrType::FloatPtr);
                        let mut dimension_vec = vec![];
                        let mut dimension_vec_in = vec![];
                        for exp in vec {
                            dimension_vec.push(exp.process(Type::Int, kit_mut).unwrap());
                        }
                        dimension_vec_in.push(-1);
                        for (inst, dimension) in dimension_vec {
                            match dimension {
                                ExpValue::Int(i) => {
                                    dimension_vec_in.push(i);
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        kit_mut.context_mut.add_var(
                            id,
                            Type::Float,
                            true,
                            true,
                            Some(param),
                            None,
                            dimension_vec_in,
                        );
                        //这里
                        kit_mut.context_mut.update_var_scope_now(id, param);
                        Ok((id.clone(), param))
                    }
                }
            }
            // BType::Int => {}
            // BType::Float => {}
            // todo!();
            // },
            FuncFParam::NonArray((tp, id)) => match tp {
                BType::Int => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Int);
                    kit_mut
                        .context_mut
                        .add_var(id, Type::Int, false, true, None, None, Vec::new());
                    //这里
                    kit_mut.context_mut.update_var_scope_now(id, param);
                    Ok((id.clone(), param))
                }
                BType::Float => {
                    let param = kit_mut.pool_inst_mut.make_param(IrType::Float);
                    kit_mut.context_mut.add_var(
                        id,
                        Type::Float,
                        false,
                        true,
                        None,
                        None,
                        Vec::new(),
                    );
                    kit_mut.context_mut.update_var_scope_now(id, param);
                    Ok((id.clone(), param))
                }
            },
        }
    }
}
impl Process for Block {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        kit_mut.context_mut.add_layer();
        for item in &mut self.block_vec {
            item.process(input, kit_mut);
        }
        kit_mut.context_mut.delete_layer();
        Ok(1)
    }
}

impl Process for BlockItem {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            BlockItem::Decl(decl) => {
                decl.process(1, kit_mut);
                return Ok(1);
            }
            BlockItem::Stmt(stmt) => {
                stmt.process(input, kit_mut);
                return Ok(1);
            }
        }
        todo!();
    }
}
impl Process for Stmt {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Stmt::Assign(assign) => {
                assign.process(input, kit_mut);
                Ok(1)
            }
            Stmt::ExpStmt(exp_stmt) => {
                exp_stmt.process((Type::Int, input.0, input.1), kit_mut); //这里可能有问题
                Ok(1)
            }
            Stmt::Block(blk) => {
                blk.process(input, kit_mut);
                Ok(1)
            }
            Stmt::If(if_stmt) => {
                if_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::While(while_stmt) => {
                while_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Break(break_stmt) => {
                break_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Continue(continue_stmt) => {
                continue_stmt.process(input, kit_mut);
                Ok(1)
            }
            Stmt::Return(ret_stmt) => {
                ret_stmt.process(input, kit_mut);
                Ok(1)
            }
        }
        // todo!();
    }
}

impl Process for Assign {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let lval = &mut self.lval;
        let symbol = kit_mut.get_var_symbol(&lval.id).unwrap();
        // let (_,symbol) = kit_mut.get_var(&lval.id).unwrap();
        // println!("assign stmt");
        let mut mes = Type::Int;
        match symbol.tp {
            Type::ConstFloat => {
                mes = Type::Float;
            }
            Type::ConstInt => {
                mes = Type::Int;
            }
            Type::Float => {
                mes = Type::Float;
            }
            Type::Int => {
                mes = Type::Int;
            }
        }

        // println!("zhe");
        let (inst_r, _) = self.exp.process(mes, kit_mut).unwrap();
        // if symbol.is_param {
        //     kit_mut.param_used(&lval.id);
        //     kit_mut
        //         .context_mut
        //         .update_var_scope_now(&self.lval.id, inst_r);
        //     return Ok(1);
        // }//函数参数不再单独处理
        if let Some(array_inst) = symbol.array_inst {
            //如果是数组
            // if symbol.layer < 0 {
            //全局变量
            let inst_offset = offset_calculate(&lval.id, &mut lval.exp_vec, kit_mut);
            let inst_ptr = kit_mut.pool_inst_mut.make_gep(array_inst, inst_offset);
            // kit_mut.context_mut.push_inst_bb(inst_offset); //这里需要吗  不需要
            kit_mut.context_mut.push_inst_bb(inst_ptr);
            match symbol.tp {
                Type::ConstFloat | Type::Float => {
                    let inst_store = kit_mut.pool_inst_mut.make_float_store(inst_ptr, inst_r);
                    kit_mut.context_mut.push_inst_bb(inst_store);
                }
                Type::ConstInt | Type::Int => {
                    let inst_store = kit_mut.pool_inst_mut.make_int_store(inst_ptr, inst_r);
                    kit_mut.context_mut.push_inst_bb(inst_store);
                }
            }

            // }
            kit_mut
                .context_mut
                .update_var_scope_now(&self.lval.id, inst_r);
            Ok(1)
            // Ok(1)
        } else {
            //不是数组
            if let Some(global_inst) = symbol.global_inst {
                //全局变量
                match symbol.tp {
                    Type::ConstFloat | Type::Float => {
                        let inst_store =
                            kit_mut.pool_inst_mut.make_float_store(global_inst, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_store = kit_mut.pool_inst_mut.make_int_store(global_inst, inst_r);
                        kit_mut.context_mut.push_inst_bb(inst_store);
                    }
                }
            }
            kit_mut
                .context_mut
                .update_var_scope_now(&self.lval.id, inst_r);
            Ok(1)
        }

        // kit_mut
        //     .context_mut
        //     .update_var_scope_now(&self.lval.id, inst_r);
        // Ok(1)
    }
}
impl Process for ExpStmt {
    type Ret = i32;
    type Message = (Type, Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        Ok(1)
    }
}

impl Process for If {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let (inst_cond, val_cond) = self.cond.process(Type::Int, kit_mut).unwrap();
        let inst_branch = kit_mut.pool_inst_mut.make_br(inst_cond);
        kit_mut.context_mut.push_inst_bb(inst_branch);
        let bb_if_name = kit_mut.context_mut.get_newbb_name();
        let inst_bb_if = kit_mut.pool_bb_mut.new_basic_block(bb_if_name.clone());
        kit_mut.context_mut.is_branch_map.insert(bb_if_name, false); //初始化
                                                                     // match kit_mut.context_mut.bb_now_mut {
                                                                     //     InfuncChoice::InFunc(bb_now) => {
                                                                     //         kit_mut
                                                                     //             .context_mut
                                                                     //             .is_branch_map
                                                                     //             .insert(bb_now.get_name().to_string(), true); //标志该块出现分支
                                                                     //     }
                                                                     //     _ => {
                                                                     //         unreachable!()
                                                                     //     }
                                                                     // }

        if let Some(stmt_else) = &mut self.else_then {
            //如果有else语句
            let bb_else_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_else = kit_mut.pool_bb_mut.new_basic_block(bb_else_name.clone());
            kit_mut
                .context_mut
                .is_branch_map
                .insert(bb_else_name, false); //初始化

            //生成一块新的bb
            let bb_successor_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_successor = kit_mut
                .pool_bb_mut
                .new_basic_block(bb_successor_name.clone());
            kit_mut
                .context_mut
                .is_branch_map
                .insert(bb_successor_name, false); //初始化

            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    // println!("下一块:{:?}", inst_bb_else.get_name());
                    // println!("下一块:{:?}", inst_bb_if.get_name());
                    bb_now.as_mut().add_next_bb(inst_bb_else); //先放判断为假的else语句
                    bb_now.as_mut().add_next_bb(inst_bb_if);
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.bb_now_set(inst_bb_else); //设置现在所在的bb块，准备归约
            stmt_else.process(input, kit_mut).unwrap(); //向该分支块内生成指令
                                                        //加一条直接跳转语句
            kit_mut
                .context_mut
                .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_mut_now是else分支的叶子交汇点
            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    // println!("下一块:{:?}", inst_bb_successor.get_name());
                    bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                }
                _ => {
                    unreachable!()
                }
            }

            // let branch_flag_else = kit_mut
            //     .context_mut
            //     .is_branch_map
            //     .get(inst_bb_else.get_name())
            //     .unwrap();
            kit_mut.context_mut.bb_now_set(inst_bb_if);
            self.then.process(input, kit_mut).unwrap();
            kit_mut
                .context_mut
                .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_now_mut是if语句块的叶子交汇点
            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    // println!("下一块:{:?}", inst_bb_successor.get_name());
                    bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.bb_now_set(inst_bb_successor); //设置现在所在的bb
        } else {
            // println!("有if没else");
            //没有else语句块
            //如果指定了该节点分支前的后继块
            //生成一块新的bb
            let bb_successor_name = kit_mut.context_mut.get_newbb_name();
            let inst_bb_successor = kit_mut
                .pool_bb_mut
                .new_basic_block(bb_successor_name.clone());
            kit_mut
                .context_mut
                .is_branch_map
                .insert(bb_successor_name, false); //初始化

            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    // println!("下一块:{:?}", inst_bb_successor.get_name());
                    // println!("下一块:{:?}", inst_bb_if.get_name());
                    bb_now.as_mut().add_next_bb(inst_bb_successor); //先放判断为假的else语句
                    bb_now.as_mut().add_next_bb(inst_bb_if);
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.bb_now_set(inst_bb_if);
            self.then.process(input, kit_mut).unwrap();
            kit_mut
                .context_mut
                .push_inst_bb(kit_mut.pool_inst_mut.make_jmp()); //bb_now_mut是if语句块的叶子交汇点

            match kit_mut.context_mut.bb_now_mut {
                InfuncChoice::InFunc(bb_now) => {
                    // println!("下一块:{:?}", inst_bb_successor.get_name());
                    bb_now.as_mut().add_next_bb(inst_bb_successor); //向if分支的叶子交汇点bb_now_mut插入下一个节点
                }
                _ => {
                    unreachable!()
                }
            }
            kit_mut.context_mut.bb_now_set(inst_bb_successor); //设置现在所在的bb
        }
        Ok(1)
    }
}
impl Process for While {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, _input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let (inst_cond, val_cond) = self.cond.process(Type::Int, kit_mut).unwrap();
        let inst_branch = kit_mut.pool_inst_mut.make_br(inst_cond);
        kit_mut.context_mut.push_inst_bb(inst_branch); //当前basicblock中放入branch指令
        let block_while_head_name = kit_mut.context_mut.get_newbb_name();
        let block_while_head = kit_mut.pool_bb_mut.new_basic_block(block_while_head_name); //生成新的块(false)
        let block_false_name = kit_mut.context_mut.get_newbb_name();
        let block_false = kit_mut.pool_bb_mut.new_basic_block(block_false_name); //生成新的块(false)
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                bb_now.as_mut().add_next_bb(block_false);
                bb_now.as_mut().add_next_bb(block_while_head);
            }
            _ => {
                unreachable!()
            }
        }
        kit_mut.context_mut.bb_now_set(block_while_head); //设置当前basicblock
                                                          // println!("while_body process starts");
        self.body
            .process((Some(block_while_head), Some(block_false)), kit_mut)
            .unwrap(); //在块内生成指令
                       // println!("while_body process finished");
        let (inst_cond, val_cond) = self.cond.process(Type::Int, kit_mut).unwrap(); //当前块中放入cond
                                                                                    // println!("cond process finished");
        let inst_branch = kit_mut.pool_inst_mut.make_br(inst_cond);
        kit_mut.context_mut.push_inst_bb(inst_branch); //当前basicblock中放入branch指令
        match kit_mut.context_mut.bb_now_mut {
            //当前块是while_body所有叶子节点的交汇点
            InfuncChoice::InFunc(bb_now) => {
                // println!("下一个节点:{:?}", block_false.get_name());
                // println!("下一个节点:{:?}", block_while_head.get_name());
                bb_now.as_mut().add_next_bb(block_false);
                bb_now.as_mut().add_next_bb(block_while_head);
            }
            _ => {
                unreachable!()
            }
        }
        // println!("set");
        kit_mut.context_mut.bb_now_set(block_false);
        // todo!()
        Ok(1)
    }
}

impl Process for Break {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
        kit_mut.context_mut.push_inst_bb(inst_jmp);
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                let (_, false_opt) = input;
                if let Some(bb_false) = false_opt {
                    bb_now.as_mut().add_next_bb(bb_false);
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(1)
    }
}
impl Process for Continue {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let inst_jmp = kit_mut.pool_inst_mut.make_jmp();
        kit_mut.context_mut.push_inst_bb(inst_jmp);
        match kit_mut.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb_now) => {
                let (head_opt, _) = input;
                if let Some(bb_false) = head_opt {
                    bb_now.as_mut().add_next_bb(bb_false);
                }
            }
            _ => {
                unreachable!()
            }
        }
        Ok(1)
    }
}

impl Process for Return {
    type Ret = i32;
    type Message = (Option<ObjPtr<BasicBlock>>, Option<ObjPtr<BasicBlock>>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if let Some(exp) = &mut self.exp {
            let (inst, val) = exp.process(Type::Int, kit_mut).unwrap(); //这里可能有问题
            match val {
                ExpValue::Float(f) => {
                    let inst_float = kit_mut.pool_inst_mut.make_float_const(f);
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst_float);
                    kit_mut.context_mut.push_inst_bb(inst_float);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                }
                ExpValue::Int(i) => {
                    let inst_int = kit_mut.pool_inst_mut.make_int_const(i);
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst_int);
                    kit_mut.context_mut.push_inst_bb(inst_int);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                }
                ExpValue::None => {
                    let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
                    kit_mut.context_mut.push_inst_bb(ret_inst);
                }
                _ => {
                    unreachable!()
                }
            }
            // let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
            // kit_mut.context_mut.push_inst_bb(ret_inst);
            Ok(1)
        } else {
            // let ret_inst = kit_mut.pool_inst_mut.make_return(inst);
            // kit_mut.context_mut.push_inst_bb(ret_inst);
            // Ok(1)
            todo!()
        }
    }
}
impl Process for Exp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for Cond {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.l_or_exp.process(input, kit_mut)
    }
}
impl Process for LVal {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // let id = self.id;
        // let mut exp_vec = &self.exp_vec;

        let (mut var, mut symbol) = (
            kit_mut.pool_inst_mut.make_int_const(0),
            Symbol {
                tp: Type::Int,
                is_array: false,
                is_param: false,
                array_inst: None,
                global_inst: None,
                layer: -2,
                dimension: vec![],
            },
        );
        let sym_tmp = kit_mut.get_var_symbol(&self.id).unwrap();
        if self.exp_vec.is_empty() {
            //如果为空
            if sym_tmp.dimension.len() > self.exp_vec.len() {
                // println!("{:?}取指针", &self.id);
                (var, symbol) = kit_mut.get_var(&self.id, None, true).unwrap();
            } else {
                // println!("{:?}不取指针", &self.id);
                (var, symbol) = kit_mut.get_var(&self.id, None, false).unwrap();
            }
            // (var, symbol) = kit_mut.get_var(&self.id, None).unwrap();
        } else {
            let sym = kit_mut.get_var_symbol(&self.id).unwrap(); //获得符号表
            let dimension_vec = sym.dimension.clone(); //获得维度信息
            let mut index = 1;
            let mut inst_base_vec = vec![];
            for _ in dimension_vec {
                let mut after = 1;
                for i in index..sym.dimension.len() {
                    after = after * sym.dimension[i]; //计算该维度每加1对应多少元素
                }
                index = index + 1;
                inst_base_vec.push(after as i32);
                //vec存储维度base信息
            }
            let mut inst_add_vec = vec![];
            let mut imm_flag = true;
            index = 0;
            for exp in &mut self.exp_vec {
                let (inst_exp, val) = exp.process(input, kit_mut).unwrap();
                match val {
                    ExpValue::Int(i) => {
                        let inst_base_now =
                            kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                        let inst_oprand = kit_mut
                            .pool_inst_mut
                            .make_int_const(i * inst_base_vec[index]);
                        inst_add_vec.push((
                            inst_base_now,
                            inst_oprand,
                            ExpValue::Int(i * inst_base_vec[index]),
                        ))
                        //将inst和数push入vec中
                    }
                    ExpValue::Float(f) => {
                        unreachable!()
                    }
                    ExpValue::None => {
                        imm_flag = false; //设置flag,表示总偏移不再是一个可以确认的值
                        let inst_base_now =
                            kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                        let inst_oprand = kit_mut.pool_inst_mut.make_mul(inst_exp, inst_base_now); //构造这一维的偏移的inst
                        inst_add_vec.push((inst_base_now, inst_oprand, ExpValue::None));
                        //将inst和数push入vec中
                    }
                    _ => {
                        unreachable!()
                    }
                }
                index = index + 1;
            }
            let mut inst_offset = kit_mut.pool_inst_mut.make_int_const(-1129);
            let mut inst_base_now = kit_mut.pool_inst_mut.make_int_const(-1129);
            if imm_flag {
                //总偏移是一个可以计算出的值
                let mut offset_final = 0;
                for (_, _, add_val) in inst_add_vec {
                    match add_val {
                        ExpValue::Int(i) => {
                            offset_final = offset_final + i;
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                // println!("偏移:{:?}",offset_final);
                inst_offset = kit_mut.pool_inst_mut.make_int_const(offset_final);
                kit_mut.context_mut.push_inst_bb(inst_offset);
            } else {
                //总偏移不是是一个可以计算出的值
                (inst_base_now, inst_offset, _) = inst_add_vec[0];
                kit_mut.context_mut.push_inst_bb(inst_base_now);
                kit_mut.context_mut.push_inst_bb(inst_offset);
                for i in 1..inst_add_vec.len() {
                    kit_mut.context_mut.push_inst_bb(inst_add_vec[i].0); //每一维的基数被push进basicblock中
                    kit_mut.context_mut.push_inst_bb(inst_add_vec[i].1); //被加数push进basicblock中
                    inst_offset = kit_mut
                        .pool_inst_mut
                        .make_add(inst_offset, inst_add_vec[i].1); //构造新的add指令，左操作数改变
                    kit_mut.context_mut.push_inst_bb(inst_offset); //add指令push进basicblock中
                }
            }

            // (var, symbol) = kit_mut.get_var(&self.id, Some(inst_offset)).unwrap();
            if sym_tmp.dimension.len() > self.exp_vec.len() {
                (var, symbol) = kit_mut.get_var(&self.id, Some(inst_offset), true).unwrap();
            } else {
                (var, symbol) = kit_mut.get_var(&self.id, Some(inst_offset), false).unwrap();
            }
        }
        // println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_ir_type());
        // println!("var_name:{:?},ir_type:{:?}",&self.id,var.as_ref().get_kind());
        match input {
            Type::ConstFloat | Type::Float => {
                match symbol.tp {
                    Type::Int | Type::ConstInt => {
                        let inst_trans = kit_mut.pool_inst_mut.make_int_to_float(var);
                        // let mut val = var.as_ref().get_float_bond();
                        // let mut val_ret = ExpValue::Int(val as i32);
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;
                                match var.as_ref().get_ptr().as_ref().get_kind() {
                                    InstKind::GlobalConstInt(i) => {
                                        val_ret = ExpValue::Float(i as f32);
                                    }
                                    _ => {}
                                }
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                            _ => {
                                let mut val_ret = ExpValue::None;
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_int_bond();
                                //     val_ret = ExpValue::Float(val as f32);
                                // }

                                match var.as_ref().get_kind() {
                                    InstKind::ConstInt(i)
                                    | InstKind::GlobalInt(i)
                                    | InstKind::GlobalConstInt(i) => {
                                        val_ret = ExpValue::Float(i as f32);
                                    }
                                    _ => {}
                                }
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                        }
                    }
                    _ => {
                        // let mut val = var.as_ref().get_float_bond();
                        // let val_ret = ExpValue::Float(val);
                        // return Ok((var,val_ret));

                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;

                                match var.as_ref().get_ptr().as_ref().get_kind() {
                                    InstKind::GlobalConstFloat(f) => {
                                        val_ret = ExpValue::Float(f);
                                    }
                                    _ => {}
                                }

                                return Ok((var, val_ret));
                            }
                            _ => {
                                // let mut val = var.as_ref().get_float_bond();
                                // let mut val_ret = ExpValue::Float(val);
                                let mut val_ret = ExpValue::None;

                                match var.as_ref().get_kind() {
                                    InstKind::ConstFloat(f)
                                    | InstKind::GlobalFloat(f)
                                    | InstKind::GlobalConstFloat(f) => {
                                        val_ret = ExpValue::Float(f);
                                    }
                                    _ => {}
                                }
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_float_bond();
                                //     val_ret = ExpValue::Float(val);
                                // }
                                // kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((var, val_ret));
                            }
                        }
                    }
                }
            }
            Type::ConstInt | Type::Int => {
                match symbol.tp {
                    Type::Float | Type::ConstFloat => {
                        let inst_trans = kit_mut.pool_inst_mut.make_float_to_int(var);
                        // let mut val = var.as_ref().get_float_bond();
                        // let mut val_ret = ExpValue::Int(val as i32);
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;
                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                match var.as_ref().get_ptr().as_ref().get_kind() {
                                    InstKind::GlobalConstFloat(f) => {
                                        val_ret = ExpValue::Int(f as i32);
                                    }
                                    _ => {}
                                }
                                return Ok((inst_trans, val_ret));
                            }
                            _ => {
                                let mut val_ret = ExpValue::None;
                                // if kit_mut.context_mut.get_layer()<0{
                                //     let val = var.as_ref().get_float_bond();
                                //     val_ret = ExpValue::Int(val as i32);
                                // }

                                match var.as_ref().get_kind() {
                                    InstKind::ConstFloat(f)
                                    | InstKind::GlobalFloat(f)
                                    | InstKind::GlobalConstFloat(f) => {
                                        val_ret = ExpValue::Int(f as i32);
                                    }
                                    _ => {}
                                }

                                kit_mut.context_mut.push_inst_bb(inst_trans);
                                return Ok((inst_trans, val_ret));
                            }
                        }
                    }
                    _ => {
                        match var.as_ref().get_kind() {
                            InstKind::Load => {
                                let mut val_ret = ExpValue::None;

                                match var.as_ref().get_ptr().as_ref().get_kind() {
                                    InstKind::GlobalConstInt(i) => {
                                        val_ret = ExpValue::Int(i);
                                    }
                                    _ => {}
                                }

                                return Ok((var, val_ret));
                            }
                            _ => {
                                // println!("var:{:?},var_type:{:?}",var.as_ref().get_kind(),var.as_ref().get_ir_type());
                                let mut val_ret = ExpValue::None;
                                match var.as_ref().get_kind() {
                                    InstKind::ConstInt(i)
                                    | InstKind::GlobalInt(i)
                                    | InstKind::GlobalConstInt(i) => {
                                        val_ret = ExpValue::Int(i);
                                    }
                                    _ => {}
                                }

                                return Ok((var, val_ret));
                            }
                        }
                    }
                }
            }
        }
        // if symbol.is_array {
        //     todo!();
        // } else {
        //     return Ok(var);
        // }
    }
}

impl Process for PrimaryExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            PrimaryExp::Exp(exp) => exp.process(input, kit_mut),
            PrimaryExp::LVal(lval) => lval.process(input, kit_mut),
            PrimaryExp::Number(num) => num.process(input, kit_mut),
        }
        // todo!();
    }
}
impl Process for Number {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            Number::FloatConst(f) => {
                // if let Some(inst) = kit_mut.context_mut.get_const_float(*f) {
                //     // println!("找到：{:?}", f);
                //     return Ok((inst, ExpValue::Float(*f)));
                // } else {
                //     let inst = kit_mut.pool_inst_mut.make_float_const(*f);
                //     kit_mut.context_mut.add_const_float(*f, inst);
                //     // println!("没找到：{:?}", f);
                //     return Ok((inst, ExpValue::Float(*f)));
                // }

                match input {
                    Type::ConstInt | Type::Int => {
                        let f = *f as i32;
                        if let Some(inst) = kit_mut.context_mut.get_const_int(f) {
                            // println!("找到：{:?}", f);
                            return Ok((inst, ExpValue::Int(f)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_int_const(f);
                            kit_mut.context_mut.add_const_int(f, inst);
                            // println!("没找到：{:?}", f);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Int(f)));
                        }
                    }
                    // Type::Float =>{

                    // }
                    Type::ConstFloat | Type::Float => {
                        if let Some(inst) = kit_mut.context_mut.get_const_float(*f) {
                            // println!("找到：{:?}", i);
                            return Ok((inst, ExpValue::Float(*f)));
                        } else {
                            // println!("没找到常量:{:?}",i);
                            let inst = kit_mut.pool_inst_mut.make_float_const(*f);
                            kit_mut.context_mut.add_const_float(*f, inst);
                            // println!("没找到：{:?}", i);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Float(*f)));
                        }
                    } // Type::Int =>{

                      // }
                }
            }
            Number::IntConst(i) => {
                match input {
                    Type::ConstFloat | Type::Float => {
                        let f = *i as f32;
                        if let Some(inst) = kit_mut.context_mut.get_const_float(f) {
                            // println!("找到：{:?}", f);
                            return Ok((inst, ExpValue::Float(f)));
                        } else {
                            let inst = kit_mut.pool_inst_mut.make_float_const(f);
                            kit_mut.context_mut.add_const_float(f, inst);
                            // println!("没找到：{:?}", f);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Float(f)));
                        }
                    }
                    // Type::Float =>{

                    // }
                    Type::ConstInt | Type::Int => {
                        if let Some(inst) = kit_mut.context_mut.get_const_int(*i) {
                            // println!("找到：{:?}", i);
                            return Ok((inst, ExpValue::Int(*i)));
                        } else {
                            // println!("没找到常量:{:?}",i);
                            let inst = kit_mut.pool_inst_mut.make_int_const(*i);
                            kit_mut.context_mut.add_const_int(*i, inst);
                            // println!("没找到：{:?}", i);
                            // println!("intconst:{}", i);
                            return Ok((inst, ExpValue::Int(*i)));
                        }
                    } // Type::Int =>{

                      // }
                }
            }
        }
    }
}

impl Process for OptionFuncRParams {
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = (Vec<Type>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        if let Some(rparams) = &mut self.func_fparams {
            Ok(rparams.process(input, kit_mut).unwrap())
        } else {
            Ok(vec![])
        }
    }
}
impl Process for UnaryExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            UnaryExp::PrimaryExp(primaryexp) => primaryexp.process(input, kit_mut),
            UnaryExp::OpUnary((unaryop, unaryexp)) => match unaryop {
                UnaryOp::Add => {
                    let (mut inst_u, mut val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_pos(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = val;
                    Ok((inst, val_ret))
                }
                UnaryOp::Minus => {
                    let (mut inst_u, mut val) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_neg(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = val;
                    match val {
                        ExpValue::Float(f) => {
                            val_ret = ExpValue::Float(-f);
                        }
                        ExpValue::Int(i) => {
                            val_ret = ExpValue::Int(-i);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
                }
                UnaryOp::Exclamation => {
                    let (inst_u, _) = unaryexp.as_mut().process(input, kit_mut).unwrap();
                    let inst = kit_mut.pool_inst_mut.make_not(inst_u);
                    kit_mut.context_mut.push_inst_bb(inst);
                    Ok((inst, ExpValue::None))
                }
            },
            UnaryExp::FuncCall((funcname, funcparams)) => {
                let inst_func = kit_mut.context_mut.module_mut.get_function(&funcname);
                let fparams = inst_func.as_ref().get_parameter_list();
                let mut fparams_type_vec = vec![];
                for fp in fparams {
                    //获得各参数类型
                    match fp.as_ref().get_ir_type() {
                        IrType::Float => {
                            fparams_type_vec.push(Type::Float);
                        }
                        IrType::Int | IrType::FloatPtr | IrType::IntPtr => {
                            //这里可能得改
                            fparams_type_vec.push(Type::Int);
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                match inst_func.as_ref().get_return_type() {
                    //根据返回值类型生成call指令
                    IrType::Float => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap(); //获得实参
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_float_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Int => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_int_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    IrType::Void => {
                        let mut args = funcparams.process(fparams_type_vec, kit_mut).unwrap();
                        let mut fname = " ".to_string();
                        if let Some((funcname_in, _)) = kit_mut
                            .context_mut
                            .module_mut
                            .function
                            .get_key_value(funcname)
                        {
                            fname = funcname_in.clone();
                        }
                        let inst = kit_mut.pool_inst_mut.make_void_call(fname, args);
                        kit_mut.context_mut.push_inst_bb(inst);
                        Ok((inst, ExpValue::None)) //这里可以进一步对返回值进行分析
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Process for UnaryOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for FuncRParams {
    type Ret = Vec<ObjPtr<Inst>>;
    type Message = (Vec<Type>);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        // match self{
        //     FuncFParams
        // }
        let mut vec = vec![];
        let mut index = 0;
        for i in &mut self.exp_vec {
            let (inst, _) = i.process(input[index], kit_mut).unwrap();
            vec.push(inst);
            index = index + 1;
        }
        Ok(vec)
    }
}

impl Process for MulExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            MulExp::UnaryExp(unaryexp) => unaryexp.process(input, kit_mut),
            MulExp::MulExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_mul(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 * f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 * i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
            MulExp::DivExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_div(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 / f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 / i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
            MulExp::ModExp((mulexp, unaryexp)) => {
                let (inst_left, lval) = mulexp.as_mut().process(input, kit_mut).unwrap();
                let (inst_right, rval) = unaryexp.process(input, kit_mut).unwrap();
                let inst = kit_mut.pool_inst_mut.make_rem(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst);
                let mut val_ret = lval;
                match lval {
                    ExpValue::Float(f1) => match rval {
                        ExpValue::Float(f2) => {
                            val_ret = ExpValue::Float(f1 % f2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    ExpValue::Int(i1) => match rval {
                        ExpValue::Int(i2) => {
                            val_ret = ExpValue::Int(i1 % i2);
                        }
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    },
                    _ => {
                        val_ret = ExpValue::None;
                    }
                }
                Ok((inst, val_ret))
            }
        }
    }
}
// impl Process for AddOp {
//     type Ret = i32;
//     type Message = (i32);
//     fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
//         todo!();
//     }
// }

impl Process for AddExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            AddExp::MulExp(mulexp) => mulexp.as_mut().process(input, kit_mut),
            AddExp::OpExp((opexp, op, mulexp)) => match op {
                AddOp::Add => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    // println!("lvar:{:?},type:{:?},rvar:{:?},type:{:?}",inst_left.as_ref().get_kind(),inst_left.as_ref().get_ir_type(),inst_right.as_ref().get_kind(),inst_right.as_ref().get_ir_type());
                    let inst = kit_mut.pool_inst_mut.make_add(inst_left, inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 + f2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 + i2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
                }
                AddOp::Minus => {
                    let (inst_left, lval) = opexp.process(input, kit_mut).unwrap();
                    let (inst_right, rval) = mulexp.process(input, kit_mut).unwrap();
                    // let inst_right_neg = kit_mut.pool_inst_mut.make_neg(inst_right);
                    let inst = kit_mut.pool_inst_mut.make_sub(inst_left, inst_right);
                    // kit_mut.context_mut.push_inst_bb(inst_right);
                    kit_mut.context_mut.push_inst_bb(inst);
                    let mut val_ret = lval;
                    match lval {
                        ExpValue::Float(f1) => match rval {
                            ExpValue::Float(f2) => {
                                val_ret = ExpValue::Float(f1 - f2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        ExpValue::Int(i1) => match rval {
                            ExpValue::Int(i2) => {
                                val_ret = ExpValue::Int(i1 - i2);
                            }
                            _ => {
                                val_ret = ExpValue::None;
                            }
                        },
                        _ => {
                            val_ret = ExpValue::None;
                        }
                    }
                    Ok((inst, val_ret))
                }
            },
        }
    }
}
impl Process for RelOp {
    type Ret = i32;
    type Message = (i32);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        todo!();
    }
}

impl Process for RelExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let tp = self.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
        match self {
            RelExp::AddExp(addexp) => addexp.process(input, kit_mut),
            RelExp::OpExp((relexp, op, addexp)) => {
                let mut tp_in = Type::Int;
                if tp > 1 {
                    //float或floatconst
                    tp_in = Type::Float;
                }
                let (mut inst_left, val_left) = relexp.process(tp_in, kit_mut).unwrap();
                let (mut inst_right, val_right) = addexp.process(tp_in, kit_mut).unwrap();
                match val_left {
                    ExpValue::Float(f) => {
                        inst_left = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    ExpValue::Int(i) => {
                        inst_left = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        inst_right = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    ExpValue::Int(i) => {
                        inst_right = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                match op {
                    RelOp::Greater => {
                        let inst_cond = kit_mut.pool_inst_mut.make_gt(inst_left, inst_right);
                        kit_mut.context_mut.push_inst_bb(inst_cond);
                        Ok((inst_cond, ExpValue::None)) //这里可能可以优化，只考虑左操作数和右操作数只有一个的情况
                    }
                    RelOp::GreaterOrEqual => {
                        let inst_cond = kit_mut.pool_inst_mut.make_ge(inst_left, inst_right);
                        kit_mut.context_mut.push_inst_bb(inst_cond);
                        Ok((inst_cond, ExpValue::None))
                    }
                    RelOp::Less => {
                        let inst_cond = kit_mut.pool_inst_mut.make_lt(inst_left, inst_right);
                        kit_mut.context_mut.push_inst_bb(inst_cond);
                        Ok((inst_cond, ExpValue::None))
                    }
                    RelOp::LessOrEqual => {
                        let inst_cond = kit_mut.pool_inst_mut.make_le(inst_left, inst_right);
                        kit_mut.context_mut.push_inst_bb(inst_cond);
                        Ok((inst_cond, ExpValue::None))
                    }
                }
            }
        }
    }
}
impl Process for EqExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type); //if中默认给Type::Int
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        let tp = self.type_process(1, kit_mut).unwrap(); //获得表达式中各比较元素应该给的类型
        match self {
            EqExp::RelExp(relexp) => relexp.process(input, kit_mut),
            EqExp::EqualExp((eqexp, relexp)) => {
                let mut tp_in = Type::Int;
                if tp > 1 {
                    tp_in = Type::Float;
                }
                let (mut inst_left, val_left) = eqexp.process(tp_in, kit_mut).unwrap();
                let (mut inst_right, val_right) = relexp.process(tp_in, kit_mut).unwrap();
                match val_left {
                    ExpValue::Float(f) => {
                        inst_left = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    ExpValue::Int(i) => {
                        inst_left = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        inst_right = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    ExpValue::Int(i) => {
                        inst_right = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                let inst_eq = kit_mut.pool_inst_mut.make_eq(inst_left, inst_right);
                // println!("push_inst_eq into bb{:?}",inst_eq.as_ref().get_kind());
                kit_mut.context_mut.push_inst_bb(inst_eq);
                Ok((inst_eq, ExpValue::None))
            }
            EqExp::NotEqualExp((eqexp, relexp)) => {
                let (mut inst_left, val_left) = eqexp.process(input, kit_mut).unwrap();
                let (mut inst_right, val_right) = relexp.process(input, kit_mut).unwrap();
                match val_left {
                    ExpValue::Float(f) => {
                        inst_left = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    ExpValue::Int(i) => {
                        inst_left = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_left);
                    }
                    _ => {}
                }
                match val_right {
                    ExpValue::Float(f) => {
                        inst_right = kit_mut.pool_inst_mut.make_float_const(f);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    ExpValue::Int(i) => {
                        inst_right = kit_mut.pool_inst_mut.make_int_const(i);
                        kit_mut.context_mut.push_inst_bb(inst_right);
                    }
                    _ => {}
                } //这里可以进一步优化,计算cond是否恒为真或假
                let inst_ne = kit_mut.pool_inst_mut.make_ne(inst_left, inst_right);
                kit_mut.context_mut.push_inst_bb(inst_ne);
                Ok((inst_ne, ExpValue::None))
            }
        }
    }
}

impl Process for LAndExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            LAndExp::EqExp(eqexp) => eqexp.process(input, kit_mut),
            LAndExp::AndExp((landexp, eqexp)) => {
                let (mut inst_left, val_left) = landexp.process(input, kit_mut).unwrap();
                let (mut inst_right, val_right) = eqexp.process(input, kit_mut).unwrap();
                let mut inst_and = kit_mut.pool_inst_mut.make_and(inst_left, inst_right);
                let mut val_and = ExpValue::None;
                // match val_left {//优化得再改改
                //     ExpValue::False =>{
                //         inst_and = kit_mut.pool_inst_mut.make_int_const(0);
                //         val_and = ExpValue::False;
                //     }
                //     ExpValue::True =>{
                //         match val_right{
                //             ExpValue::False =>{
                //                 inst_and = kit_mut.pool_inst_mut.make_int_const(0);
                //                 val_and = ExpValue::False;
                //             }
                //             ExpValue::True =>{
                //                 inst_and = kit_mut.pool_inst_mut.make_int_const(1);
                //                 val_and = ExpValue::True;
                //             }
                //             ExpValue::None =>{

                //             }
                //             _=>{
                //                 unreachable!()
                //             }
                //         }
                //     }
                //     ExpValue::None =>{
                //         match val_right{
                //             ExpValue::False =>{
                //                 inst_and = kit_mut.pool_inst_mut.make_int_const(0);
                //                 val_and = ExpValue::False;
                //             }
                //             ExpValue::True =>{

                //             }
                //             ExpValue::None =>{

                //             }
                //             _=>{
                //                 unreachable!()
                //             }
                //         }
                //     }
                //     _=>{
                //         unreachable!()
                //     }
                // }
                kit_mut.context_mut.push_inst_bb(inst_and);
                Ok((inst_and, val_and))
            }
        }
    }
}
impl Process for ConstExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        self.add_exp.process(input, kit_mut)
    }
}

impl Process for LOrExp {
    type Ret = (ObjPtr<Inst>, ExpValue);
    type Message = (Type);
    fn process(&mut self, input: Self::Message, kit_mut: &mut Kit) -> Result<Self::Ret, Error> {
        match self {
            LOrExp::LAndExp(landexp) => landexp.process(input, kit_mut),
            LOrExp::OrExp((lorexp, landexp)) => {
                let (inst_left, val_left) = lorexp.process(input, kit_mut).unwrap();
                let (inst_right, val_right) = landexp.process(input, kit_mut).unwrap();
                let mut inst_or = kit_mut.pool_inst_mut.make_or(inst_left, inst_right);
                let mut val_or = ExpValue::None;
                // match val_left {
                //     ExpValue::Int(i) =>{
                //         if i!=0{//true
                //             inst_or = kit_mut.pool_inst_mut.make_int_const(1);
                //             val_or = ExpValue::Int(1);
                //         }else{//false
                //             match val_right {
                //                 ExpValue::Int(x) =>{
                //                     if x!=0{//true
                //                         inst_or = kit_mut.pool_inst_mut.make_int_const(1);
                //                         val_or = ExpValue::Int(1);
                //                     }else{
                //                         inst_or = kit_mut.pool_inst_mut.make_int_const(0);
                //                         val_or = ExpValue::Int(0);
                //                     }
                //                 }
                //                 ExpValue::None =>{

                //                 }
                //                 _=>{
                //                     unreachable!()
                //                 }
                //             }
                //         }

                //     }
                //     ExpValue::None=>{
                //         match val_right {
                //             ExpValue::Int(x) =>{
                //                 if x!=0{//为true
                //                     inst_or = kit_mut.pool_inst_mut.make_int_const(1);
                //                 val_or = ExpValue::Int(1);
                //                 }else{//为false

                //                 }

                //             }
                //             ExpValue::None=>{

                //             }
                //             _=>{unreachable!()}
                //         }
                //     }
                //     _=>{
                //         unreachable!()
                //     }
                // }
                kit_mut.context_mut.push_inst_bb(inst_or);
                Ok((inst_or, val_or)) //这里或许可以优化
            }
        }
    }
}

pub fn offset_calculate(id: &str, exp_vec: &mut Vec<Exp>, kit_mut: &mut Kit) -> ObjPtr<Inst> {
    let (mut var, mut symbol) = (
        kit_mut.pool_inst_mut.make_int_const(0),
        Symbol {
            tp: Type::Int,
            is_array: false,
            is_param: false,
            array_inst: None,
            global_inst: None,
            layer: -2,
            dimension: vec![],
        },
    ); //初始化

    let sym = kit_mut.get_var_symbol(id).unwrap(); //获得符号表
    let dimension_vec = sym.dimension.clone(); //获得维度信息
    let mut index = 1;
    let mut inst_base_vec = vec![];
    for _ in dimension_vec {
        let mut after = 1;
        for i in index..sym.dimension.len() {
            after = after * sym.dimension[i]; //计算该维度每加1对应多少元素
        }
        index = index + 1;
        inst_base_vec.push(after as i32);
        //vec存储维度base信息
    }
    let mut inst_add_vec = vec![];
    let mut imm_flag = true;
    index = 0;
    for exp in exp_vec {
        let (inst_exp, val) = exp.process(symbol.tp, kit_mut).unwrap();
        match val {
            ExpValue::Int(i) => {
                let inst_base_now = kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                let inst_oprand = kit_mut
                    .pool_inst_mut
                    .make_int_const(i * inst_base_vec[index]);
                inst_add_vec.push((
                    inst_base_now,
                    inst_oprand,
                    ExpValue::Int(i * inst_base_vec[index]),
                ))
                //将inst和数push入vec中
            }
            ExpValue::Float(f) => {
                unreachable!()
            }
            ExpValue::None => {
                imm_flag = false; //设置flag,表示总偏移不再是一个可以确认的值
                let inst_base_now = kit_mut.pool_inst_mut.make_int_const(inst_base_vec[index]); //构造base对应的inst
                let inst_oprand = kit_mut.pool_inst_mut.make_mul(inst_exp, inst_base_now); //构造这一维的偏移的inst
                inst_add_vec.push((inst_base_now, inst_oprand, ExpValue::None));
                //将inst和数push入vec中
            }
            _ => {
                unreachable!()
            }
        }
        index = index + 1;
    }
    let mut inst_offset = kit_mut.pool_inst_mut.make_int_const(-1129);
    let mut inst_base_now = kit_mut.pool_inst_mut.make_int_const(-1129);
    if imm_flag {
        //总偏移是一个可以计算出的值
        let mut offset_final = 0;
        for (_, _, add_val) in inst_add_vec {
            match add_val {
                ExpValue::Int(i) => {
                    offset_final = offset_final + i;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        inst_offset = kit_mut.pool_inst_mut.make_int_const(offset_final);
        kit_mut.context_mut.push_inst_bb(inst_offset);
    } else {
        //总偏移不是是一个可以计算出的值
        (inst_base_now, inst_offset, _) = inst_add_vec[0];
        kit_mut.context_mut.push_inst_bb(inst_base_now);
        kit_mut.context_mut.push_inst_bb(inst_offset);
        for i in 1..inst_add_vec.len() {
            kit_mut.context_mut.push_inst_bb(inst_add_vec[i].0); //每一维的基数被push进basicblock中
            kit_mut.context_mut.push_inst_bb(inst_add_vec[i].1); //被加数push进basicblock中
            inst_offset = kit_mut
                .pool_inst_mut
                .make_add(inst_offset, inst_add_vec[i].1); //构造新的add指令，左操作数改变
            kit_mut.context_mut.push_inst_bb(inst_offset); //add指令push进basicblock中
        }
    }
    // println!("左值偏移:{:?}",inst_offset.as_ref().get_kind());
    // (var, symbol) = kit_mut.get_var(&self.id).unwrap();

    inst_offset
}
