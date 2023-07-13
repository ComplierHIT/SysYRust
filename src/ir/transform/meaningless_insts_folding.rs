use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{BinOp, Inst, InstKind},
        module::Module,
        tools::{bfs_inst_process, func_process, replace_inst}, ir_type::IrType,
    },
    utility::{ObjPool, ObjPtr},
};

pub fn meaningless_inst_folding(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |_, func| {
        bfs_inst_process(func.get_head(), |inst| delete_useless_inst(inst, pools.1))
    });
}

pub fn delete_useless_inst(inst: ObjPtr<Inst>, pool: &mut ObjPool<Inst>) {
    match inst.get_kind() {
        InstKind::Binary(binop) => match binop {
            BinOp::Add | BinOp::Sub => {
                let operands = inst.get_operands();
                match operands[0].get_kind() {
                    InstKind::ConstInt(i) => {
                        if i == 0 {
                            replace_inst(inst, operands[1]);
                        }
                    }
                    _ => match operands[1].get_kind() {
                        InstKind::ConstInt(i) => {
                            if i == 0 {
                                replace_inst(inst, operands[0]);
                            }
                        }
                        _ => {
                            
                        }
                    },
                }
            }
            BinOp::Mul | BinOp::Div => {
                let operands = inst.get_operands();
                match operands[0].get_kind() {
                    InstKind::ConstInt(i) => {
                        if i == 1 {
                            replace_inst(inst, operands[1]);
                        }else if i==-1{
                            let inst_new = pool.make_neg(operands[1]);
                            inst.as_mut().insert_before(inst_new);
                            replace_inst(inst, inst_new);
                        }
                    }
                    _ => match operands[1].get_kind() {
                        InstKind::ConstInt(i) => {
                            if i == 1 {
                                replace_inst(inst, operands[0]);
                            }else if i==-1{
                                let inst_new = pool.make_neg(operands[0]);
                                inst.as_mut().insert_before(inst_new);
                                replace_inst(inst, inst_new);
                            }
                        }
                        _ => {}
                    },
                }
            }
            BinOp::Eq | BinOp::Le | BinOp::Ge => {
                let operands = inst.get_operands();
                if operands[0] == operands[1] {
                    let inst_new = pool.make_int_const(1);
                    inst.as_mut().insert_before(inst_new);
                    replace_inst(inst, inst_new);
                }
            }
            BinOp::Gt | BinOp::Lt | BinOp::Ne => {
                let operands = inst.get_operands();
                if operands[0] == operands[1] {
                    let inst_new = pool.make_int_const(0);
                    inst.as_mut().insert_before(inst_new);
                    replace_inst(inst, inst_new);
                }
            }
            _ => {}
        },
        InstKind::Load =>{
            let operand = inst.get_operands();
            match operand[0].get_kind() {
                InstKind::GlobalConstInt(i) =>{
                    let inst_new = pool.make_int_const(i);
                    inst.as_mut().insert_before(inst_new);
                    replace_inst(inst, inst_new);
                }
                InstKind::GlobalConstFloat(f) =>{
                    let inst_new = pool.make_float_const(f);
                    inst.as_mut().insert_before(inst_new);
                    replace_inst(inst, inst_new);
                }
                _=>{}
            }
        }
        _ => {}
    }
}
