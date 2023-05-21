///! 本文件为分支指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 创建条件跳转指令
    /// # Arguments
    /// * `cond` - 条件
    /// # Returns
    /// 返回创建的条件跳转指令
    pub fn make_br(&mut self, cond: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let ir_type = IrType::Void;
        let kind = InstKind::Branch;
        let operands = vec![cond];
        let inst = Inst::new(ir_type, kind, operands);
        let inst = self.put(inst);

        // 设置use_list
        cond.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 创建无条件跳转指令
    /// # Returns
    /// 返回创建的无条件跳转指令
    pub fn make_jmp(&mut self) -> ObjPtr<Inst> {
        let ir_type = IrType::Void;
        let kind = InstKind::Branch;
        let operands = vec![];
        let mut inst = Inst::new(ir_type, kind, operands);
        self.put(inst)
    }
}

impl Inst {
    /// 判断是否为条件跳转指令
    pub fn is_br(&self) -> bool {
        match self.kind {
            InstKind::Branch => self.user.get_operands_size() == 1,
            _ => panic!("InstKind is not Branch"),
        }
    }

    /// 判断是否为无条件跳转指令
    pub fn is_jmp(&self) -> bool {
        match self.kind {
            InstKind::Branch => self.user.get_operands_size() == 0,
            _ => panic!("InstKind is not Branch"),
        }
    }

    /// 获得条件跳转指令的条件
    pub fn get_br_cond(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }

    /// 设置条件跳转指令的条件
    /// # Arguments
    /// * `cond` - 条件
    pub fn set_br_cond(&mut self, cond: ObjPtr<Inst>) {
        // 修改use_list
        self.user.get_operand(0).as_mut().remove_user(self);
        cond.as_mut().add_user(self);

        self.user.set_operand(0, cond);
    }
}
