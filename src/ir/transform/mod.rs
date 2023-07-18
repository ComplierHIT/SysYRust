use super::{basicblock::BasicBlock, instruction::Inst, module::Module};
use super::{dump_now, tools::*};
use crate::utility::ObjPool;

mod array_transform;
mod constant_folding;
mod dead_code_eliminate;
mod delete_redundant_load_store;
mod func_inline;
mod global_value_numbering;
mod global_var_transform;
mod loop_operation;
mod meaningless_insts_folding;
mod phi_optimizer;
mod return_unused;
mod simplify_cfg;
mod verify;

pub fn optimizer_run(
    module: &mut Module,
    mut pools: (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module, &mut pools, optimize_flag);

    if optimize_flag {
        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 循环优化
        loop_operation::loop_optimize(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);
        simplify_cfg::simplify_cfg_run(module, &mut pools);

        // 函数内联
        func_inline::inline_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // TODO: 性能优化

        // 再做一次
        functional_optimizer(module, &mut pools, optimize_flag);
    }
}

fn functional_optimizer(
    module: &mut Module,
    mut pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    debug_assert!(verify::verify_run(module));

    // phi优化
    phi_optimizer::phi_run(module);

    // 常量折叠
    constant_folding::constant_folding(module, &mut pools, optimize_flag);
    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, optimize_flag);

    // 消除不必要的指令
    meaningless_insts_folding::meaningless_inst_folding(module, &mut pools);

    // gvn
    global_value_numbering::gvn(module, optimize_flag);

    // 常量折叠
    constant_folding::constant_folding(module, &mut pools, optimize_flag);

    // 数组转换
    array_transform::array_transform(module, &mut pools);
    // 数组重新初始化
    array_transform::array_init_optimize(module);
    // 数组第一次load优化
    array_transform::array_first_load_optimize(module, &mut pools);

    // 全局变量转换
    global_var_transform::global_var_transform(module, &mut pools);

    // 函数返回值优化
    return_unused::return_unused(module);

    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, true);
    // 全局死代码删除
    dead_code_eliminate::global_eliminate(module);
}
