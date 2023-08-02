use rand::seq::index;

use super::*;

/// handle spill v3实现
impl Func {
    ///为handle spill 计算寄存器活跃区间
    /// 会认为zero,ra,sp,tp,gp在所有块中始终活跃
    pub fn calc_live_for_handle_spill(&self) {
        self.calc_live_base();
        //把sp和ra寄存器加入到所有的块的live out,live in中，表示这些寄存器永远不能在函数中自由分配使用
        for bb in self.blocks.iter() {
            //0:zero, 1:ra, 2:sp 3:gp 4:tp
            for id in 0..=4 {
                bb.as_mut().live_in.insert(Reg::new(id, ScalarType::Int));
                bb.as_mut().live_out.insert(Reg::new(id, ScalarType::Int));
            }
            //加入s0,避免在handle spill中使用了s0
            bb.as_mut().live_in.insert(Reg::new(8, ScalarType::Int));
            bb.as_mut().live_out.insert(Reg::new(8, ScalarType::Int));
        }
    }

    ///精细化的handle spill:
    ///
    ///遇到spilling寄存器的时候:
    /// * 优先使用available的寄存器
    ///     其中,优先使用caller save的寄存器
    ///     ,再考虑使用callee save的寄存器.
    /// * 如果要使用unavailable的寄存器,才需要进行spill操作来保存和恢复原值
    ///     优先使用caller save的寄存器,
    /// * 一定要spill到内存上的时候,使用递增的slot,把slot记录到数组的表中,等待重排
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.calc_live_for_handle_spill();
        self.build_reg_intervals();
        //先分配空间
        //对于spillings用到的空间直接一人一个
        let regs = self.draw_all_virtual_regs();
        for spilling_reg in self.reg_alloc_info.spillings.iter() {
            debug_assert!(
                regs.contains(&Reg::new(*spilling_reg, ScalarType::Int))
                    || regs.contains(&Reg::new(*spilling_reg, ScalarType::Int))
            );
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            let spilling_reg = if regs.contains(&Reg::new(*spilling_reg, ScalarType::Int)) {
                debug_assert!(!regs.contains(&Reg::new(*spilling_reg, ScalarType::Float)));
                Reg::new(*spilling_reg, ScalarType::Int)
            } else {
                debug_assert!(regs.contains(&Reg::new(*spilling_reg, ScalarType::Float)));
                Reg::new(*spilling_reg, ScalarType::Float)
            };
            debug_assert!(!self.spill_stack_map.contains_key(&spilling_reg));
            self.spill_stack_map.insert(spilling_reg, new_stack_slot);
            self.stack_addr.push_back(new_stack_slot);
        }
        // Func::print_func(ObjPtr::new(&self), "mm.txt");
        //为物理寄存器相关的借还开辟空间
        let mut phisic_mems = HashMap::new();
        for reg in Reg::get_all_not_specials() {
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            self.stack_addr.push_back(new_stack_slot);
            phisic_mems.insert(reg, new_stack_slot);
        }

        for bb in self.blocks.iter() {
            // Func::handle_spill_of_block_tmp(
            //     bb,
            //     pool,
            //     &self.reg_alloc_info.spillings,
            //     &self.spill_stack_map,
            //     &phisic_mems,
            // );
            Func::handle_spill_of_block(
                bb,
                pool,
                &self.reg_alloc_info.spillings,
                &self.spill_stack_map,
                &phisic_mems,
            );
        }
    }

    ///在handle spill之后调用
    /// 返回该函数使用了哪些callee saved的寄存器
    pub fn draw_used_callees(&self) -> HashSet<Reg> {
        let mut callees: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_callee_save() {
                        callees.insert(reg);
                    }
                }
            }
        }
        callees
    }

    /// 该函数应该在vtop之后调用
    /// 获取该函数使用到的caller save寄存器
    pub fn draw_used_callers(&self) -> HashSet<Reg> {
        let mut callers: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_caller_save() {
                        callers.insert(reg);
                    }
                }
            }
        }
        callers
    }
}

impl Func {
    ///考虑有临时寄存器可以用
    /// 该操作应该在p2v之后进行,认为遇到的虚拟寄存器都是临时寄存器
    fn handle_spill_of_block(
        bb: &ObjPtr<BB>,
        pool: &mut BackendPool,
        spillings: &HashSet<i32>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        phisic_mem: &HashMap<Reg, StackSlot>,
    ) {
        //优先使用临时寄存器,然后使用其他空余寄存器

        //维护一个表,记录当前各个物理寄存器的持有者
        let mut next_occurs: HashMap<Reg, LinkedList<(usize, bool)>> = HashMap::new();
        //初始化holder
        bb.live_in.iter().for_each(|reg| {
            next_occurs.insert(*reg, LinkedList::new());
        });
        // 维护一个物理寄存器的作用区间队列,每次的def和use压入栈中 (先压入use,再压入def)
        // 每个链表元素为(reg,if_def)
        for (index, inst) in bb.insts.iter().enumerate() {
            for reg in inst.get_reg_use() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, false));
            }
            for reg in inst.get_reg_def() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, true));
            }
        }
        bb.live_out.iter().for_each(|reg| {
            next_occurs
                .get_mut(reg)
                .unwrap()
                .push_back((bb.insts.len(), false));
        });
        //对于其他的没有加入到表中的寄存器,也添加列表
        for reg in Reg::get_all_not_specials() {
            if next_occurs.contains_key(&reg) {
                continue;
            }
            next_occurs.insert(reg, LinkedList::new());
        }

        //然后对于不在live out中的但是insts中出现的所有寄存器,直接全部加上一个超长长度,
        //并且后面设置为true是为了提示不用保存
        for (_, next_occur) in next_occurs.iter_mut() {
            next_occur.push_back((bb.insts.len() * 2, true));
        }

        //准备用于进行选择选择要借用的寄存器的函数
        let choose_borrow = |inst: &ObjPtr<LIRInst>,
                             rentor: &Reg,
                             next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
                             rentors: &HashMap<Reg, Reg>,
                             holders: &HashMap<Reg, Reg>|
         -> Reg {
            debug_assert!(!rentors.contains_key(rentor));
            //获取所有非特殊寄存器
            let mut regs = RegUsedStat::init_unspecial_regs();
            //然后禁止当前指令使用到的物理寄存器,以及当前指令涉及的虚拟寄存器使用到的寄存器
            for reg in inst.get_regs() {
                if reg.is_physic() {
                    regs.use_reg(reg.get_color())
                } else if let Some(br) = rentors.get(&reg) {
                    debug_assert!(br.is_physic());
                    regs.use_reg(br.get_color());
                }
            }

            //禁止种类不同的寄存器
            regs.merge(&RegUsedStat::init_for_reg(rentor.get_type()));
            let mut choices: Vec<(Reg, usize)> = Vec::new();
            //然后建立可用寄存器列表
            for reg in Reg::get_all_not_specials() {
                if !regs.is_available_reg(reg.get_color()) {
                    continue;
                }
                let old_holder = holders.get(&reg);
                let old_holder = if old_holder.is_some() {
                    old_holder.unwrap()
                } else {
                    &reg
                };
                let next_occur = next_occurs.get(old_holder).unwrap().front().unwrap();
                let (index, if_def) = next_occur;
                //因为def的情况代价更小更适合选,所以相同前置的情况下先设置为1,
                let next_occur = index << 1 | (if *if_def { 1 } else { 0 });
                choices.push((*old_holder, next_occur));
            }
            //对 order 进行排序
            choices.sort_by_key(|item| item.1);
            //获取该虚拟寄存器的下一次出现
            debug_assert!(choices.len() != 0);
            let to_borrow_from = choices.last().unwrap().0;
            if to_borrow_from.is_physic() {
                to_borrow_from
            } else {
                rentors.get(&to_borrow_from).unwrap().clone()
            }
        };
        let borrow = |rentor: &Reg,
                      borrowed: &Reg,
                      inst: &ObjPtr<LIRInst>,
                      next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
                      rentors: &mut HashMap<Reg, Reg>,
                      holders: &mut HashMap<Reg, Reg>,
                      pool: &mut BackendPool,
                      new_insts: &mut Vec<ObjPtr<LIRInst>>| {
            //首先判断是否需要进行寄存器的归还
            match holders.get(borrowed) {
                Some(holder) => {
                    if !holder.is_physic() {
                        //判断是否需要把该寄存器的值还回去
                        //如果下一个不是def,就需要把值归还回去
                        let next_occur = next_occurs.get(holder).unwrap().front().unwrap();
                        let if_then_def = next_occur.1;
                        debug_assert!(!if_then_def);
                        if !if_then_def {
                            let pos = spill_stack_map.get(holder).unwrap().get_pos();
                            let back_inst = LIRInst::build_storetostack_inst(borrowed, pos);
                            new_insts.push(pool.put_inst(back_inst));
                            config::record_spill(
                                "",
                                &bb.label.as_str(),
                                format!("把虚拟寄存器{}值从{}写回栈{}上", rentor, borrowed, pos,)
                                    .as_str(),
                            );
                        }
                    } else {
                        debug_assert!(holder == borrowed);
                        //对于持有者当前为物理寄存器的情况,根据下一次使用
                        let if_then_def = next_occurs.get(holder).unwrap().front().unwrap().1;
                        if !if_then_def {
                            //需要暂时保存该值到栈上,以待下次使用
                            debug_assert!(phisic_mem.contains_key(borrowed), "{}", borrowed);
                            let pos = phisic_mem.get(borrowed).unwrap().get_pos();
                            let back_inst = LIRInst::build_storetostack_inst(borrowed, pos);
                            new_insts.push(pool.put_inst(back_inst));
                            config::record_spill(
                                "",
                                &bb.label.as_str(),
                                format!("把物理寄存器{}原值暂存到栈{}上", borrowed, pos,).as_str(),
                            );
                        }
                    }
                    rentors.remove(holder);
                    holders.remove(borrowed);
                }
                None => (),
            };
            //然后判断是否需要拿回rentor寄存器原本的值
            //需要
            if inst.get_reg_use().contains(rentor) {
                let pos = spill_stack_map.get(rentor).unwrap().get_pos();
                let load_back_inst = LIRInst::build_loadstack_inst(borrowed, pos);
                new_insts.push(pool.put_inst(load_back_inst));
                config::record_spill(
                    "",
                    &bb.label.as_str(),
                    format!("从{}取回虚拟寄存器{}原值到{}", pos, rentor, borrowed).as_str(),
                );
            }
            //修改 rent hold表
            holders.insert(*borrowed, *rentor);
            rentors.insert(*rentor, *borrowed);
        };
        //寄存器归还逻辑
        let return_reg = |inst: ObjPtr<LIRInst>,
                          rentor: &Reg,
                          borrowed: &Reg,
                          next_occurs: &HashMap<Reg, LinkedList<(usize, bool)>>,
                          rentors: &mut HashMap<Reg, Reg>,
                          holders: &mut HashMap<Reg, Reg>,
                          pool: &mut BackendPool,
                          new_insts: &mut Vec<ObjPtr<LIRInst>>| {
            debug_assert!(spillings.contains(&rentor.get_id()));
            debug_assert!(rentors.get(rentor).unwrap() == borrowed);
            debug_assert!(holders.get(borrowed).unwrap() == rentor);
            //虽然但是一个块中不应该出现虚拟寄存器的两次写,so
            debug_assert!(next_occurs.get(rentor).unwrap().front().unwrap().1 != true);
            let pos = spill_stack_map.get(rentor).unwrap().get_pos();
            //把spilling寄存器的值还回栈上
            config::record_spill(
                "",
                &bb.label.as_str(),
                format!("把spilling寄存器{}值从{}写回栈{}处", rentor, borrowed, pos).as_str(),
            );
            let self_back_inst = LIRInst::build_storetostack_inst(&borrowed, pos);
            new_insts.push(pool.put_inst(self_back_inst));
            //判断是否要把物理寄存器的值取回
            let if_use = inst.get_reg_use().contains(borrowed);
            if if_use {
                let owner_pos = phisic_mem.get(&borrowed).unwrap().get_pos();
                let return_inst = LIRInst::build_loadstack_inst(&borrowed, owner_pos);
                new_insts.push(pool.put_inst(return_inst));
                config::record_spill(
                    "",
                    &bb.label.as_str(),
                    format!("取回物理寄存器{}原值", borrowed).as_str(),
                );
            }
            //更新rentor 和rentor的状态
            rentors.remove(rentor);
            holders.insert(*borrowed, *borrowed);
        };
        //归还物理寄存器的逻辑
        let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
        let mut rentors: HashMap<Reg, Reg> = HashMap::new();
        let mut holders: HashMap<Reg, Reg> = HashMap::new();
        //初始化holder
        bb.live_in.iter().for_each(|reg| {
            if reg.is_physic() {
                holders.insert(*reg, *reg);
            }
        });
        //正式分配流程,
        let mut index = 0;
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            match inst.get_type() {
                InstrsType::Branch(_) | InstrsType::Jump | InstrsType::Ret(_) => {
                    break;
                }
                _ => (),
            };
            //更新next occur表
            for reg in inst.get_regs() {
                let next_occur = next_occurs.get_mut(&reg).unwrap();
                while !next_occur.is_empty() {
                    let front = next_occur.front().unwrap();
                    if front.0 <= index {
                        next_occur.pop_front();
                        continue;
                    }
                    break;
                }
            }

            //然后归还
            for reg in inst.get_reg_use() {
                //判断是否有需要归还的寄存器 (把值取回物理寄存器,此处需要一个物理寄存器相关的空间)
                if reg.is_physic() && holders.contains_key(&reg) {
                    //遇到的物理寄存器一定有持有者
                    let holder = holders.get(&reg).unwrap();
                    if &reg != holder {
                        //如果寄存器不在当前phisicreg 手上,则进行归还
                        //统一归还操作为归还到栈空间上,无用访存指令后面会删除
                        let rentor = *holder;
                        debug_assert!(
                            next_occurs.get(&rentor).unwrap().front().unwrap().0 <= bb.insts.len()
                        );
                        return_reg(
                            *inst,
                            &rentor,
                            &reg,
                            &next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                } else if reg.is_physic() {
                    let pos = phisic_mem.get(&reg).unwrap().get_pos();
                    let load_inst = LIRInst::build_loadstack_inst(&reg, pos);
                    new_insts.push(pool.put_inst(load_inst));
                    holders.insert(reg, reg);
                }
            }

            let mut to_add_to_holder = Vec::new();
            for reg in inst.get_reg_def() {
                if reg.is_physic() && holders.contains_key(&reg) {
                    //遇到的物理寄存器一定有持有者
                    let holder = holders.get(&reg).unwrap();
                    if &reg != holder {
                        //如果寄存器不在当前phisicreg 手上,则进行归还
                        //统一归还操作为归还到栈空间上,无用访存指令后面会删除
                        let rentor = *holder;
                        debug_assert!(
                            next_occurs.get(&rentor).unwrap().front().unwrap().0 <= bb.insts.len()
                        );
                        return_reg(
                            *inst,
                            &rentor,
                            &reg,
                            &next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                } else if reg.is_physic() {
                    to_add_to_holder.push(reg);
                }
            }
            //再租借
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    if !rentors.contains_key(&reg) {
                        let to_borrow =
                            choose_borrow(inst, &reg, &mut next_occurs, &rentors, &holders);
                        borrow(
                            &reg,
                            &to_borrow,
                            inst,
                            &mut next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                }
            }
            //判断是否有需要把值存回栈上的寄存器
            //然后对该指令进行寄存器替换
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    debug_assert!(spillings.contains(&reg.get_id()));
                    let borrowed = rentors.get(&reg).unwrap();
                    inst.as_mut().replace_reg(&reg, borrowed);
                }
            }

            //然后加入替换后的指令
            new_insts.push(*inst);
            //加入物理寄存器的新持有者
            for new_holder in to_add_to_holder {
                debug_assert!(!holders.contains_key(&new_holder));
                holders.insert(new_holder, new_holder);
            }
            //根据当前只有表进行试探归还
            let mut to_relase = Vec::new();
            for (p_reg, holder) in holders.iter() {
                let next_occur = next_occurs.get(holder).unwrap();
                if holder.is_physic() {
                    debug_assert!(p_reg == holder);
                    // //如果下一个是def,可以不用持有了
                    if next_occur.front().unwrap().1 {
                        to_relase.push((*holder, *p_reg));
                    }
                    continue;
                }
                if next_occur.front().unwrap().0 > bb.insts.len() {
                    to_relase.push((*holder, *p_reg));
                    continue;
                }
                debug_assert!(
                    next_occur.front().unwrap().1 == false,
                    "{}{}",
                    p_reg,
                    holder
                );
            }
            for (rentor, borrowed) in to_relase.iter() {
                holders.remove(borrowed);
                rentors.remove(rentor);
            }

            index += 1;
        }
        //在块的最后,在跳转之前,判断是否有哪些寄存器还没有归还到主人手里,但是应该归还
        for (rentor, borrow) in rentors.iter() {
            //如果spillings寄存器值需要归还
            if bb.live_out.contains(&rentor) {
                let pos = spill_stack_map.get(&rentor).unwrap().get_pos();
                let return_inst = LIRInst::build_storetostack_inst(&borrow, pos);
                new_insts.push(pool.put_inst(return_inst));
                config::record_spill(
                    "",
                    &bb.label.as_str(),
                    format!("把虚拟寄存器{}值写回栈{}上", rentor, pos).as_str(),
                );
            }
            //如果对应物理寄存器值应该取回
            if bb.live_out.contains(&borrow) {
                let pos = phisic_mem.get(&borrow).unwrap().get_pos();
                let get_back_inst = LIRInst::build_loadstack_inst(&borrow, pos);
                new_insts.push(pool.put_inst(get_back_inst));
                config::record_spill(
                    "",
                    &bb.label.as_str(),
                    format!("从栈{}取回物理寄存器{}原值", pos, borrow).as_str(),
                );
            }
        }

        //加入最后的跳转
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            //更新next occur表
            for reg in inst.get_regs() {
                let next_occur = next_occurs.get_mut(&reg).unwrap();
                while !next_occur.is_empty() {
                    let front = next_occur.front().unwrap();
                    if front.0 <= index {
                        next_occur.pop_front();
                        continue;
                    }
                    break;
                }
            }

            //然后归还
            for reg in inst.get_reg_use() {
                //判断是否有需要归还的寄存器 (把值取回物理寄存器,此处需要一个物理寄存器相关的空间)
                if reg.is_physic() && holders.contains_key(&reg) {
                    //遇到的物理寄存器一定有持有者
                    let holder = holders.get(&reg).unwrap();
                    if &reg != holder {
                        //如果寄存器不在当前phisicreg 手上,则进行归还
                        //统一归还操作为归还到栈空间上,无用访存指令后面会删除
                        let rentor = *holder;
                        debug_assert!(
                            next_occurs.get(&rentor).unwrap().front().unwrap().0 <= bb.insts.len()
                        );
                        return_reg(
                            *inst,
                            &rentor,
                            &reg,
                            &next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                } else if reg.is_physic() {
                    let pos = phisic_mem.get(&reg).unwrap().get_pos();
                    let load_inst = LIRInst::build_loadstack_inst(&reg, pos);
                    new_insts.push(pool.put_inst(load_inst));
                    holders.insert(reg, reg);
                }
            }

            let mut to_add_to_holder = Vec::new();
            for reg in inst.get_reg_def() {
                if reg.is_physic() && holders.contains_key(&reg) {
                    //遇到的物理寄存器一定有持有者
                    let holder = holders.get(&reg).unwrap();
                    if &reg != holder {
                        //如果寄存器不在当前phisicreg 手上,则进行归还
                        //统一归还操作为归还到栈空间上,无用访存指令后面会删除
                        let rentor = *holder;
                        debug_assert!(
                            next_occurs.get(&rentor).unwrap().front().unwrap().0 <= bb.insts.len()
                        );
                        return_reg(
                            *inst,
                            &rentor,
                            &reg,
                            &next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                } else if reg.is_physic() {
                    to_add_to_holder.push(reg);
                }
            }
            //再租借
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    if !rentors.contains_key(&reg) {
                        let to_borrow =
                            choose_borrow(inst, &reg, &mut next_occurs, &rentors, &holders);
                        borrow(
                            &reg,
                            &to_borrow,
                            inst,
                            &mut next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                }
            }
            //判断是否有需要把值存回栈上的寄存器
            //然后对该指令进行寄存器替换
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    debug_assert!(spillings.contains(&reg.get_id()));
                    let borrowed = rentors.get(&reg).unwrap();
                    inst.as_mut().replace_reg(&reg, borrowed);
                }
            }

            //然后加入替换后的指令
            new_insts.push(*inst);
            //加入物理寄存器的新持有者
            for new_holder in to_add_to_holder {
                debug_assert!(!holders.contains_key(&new_holder));
                holders.insert(new_holder, new_holder);
            }
            //根据当前只有表进行试探归还
            let mut to_relase = Vec::new();
            for (p_reg, holder) in holders.iter() {
                let next_occur = next_occurs.get(holder).unwrap();
                if holder.is_physic() {
                    debug_assert!(p_reg == holder);
                    // //如果下一个是def,可以不用持有了
                    if next_occur.front().unwrap().1 {
                        to_relase.push((*holder, *p_reg));
                    }
                    continue;
                }
                if next_occur.front().unwrap().0 > bb.insts.len() {
                    to_relase.push((*holder, *p_reg));
                    continue;
                }
                debug_assert!(
                    next_occur.front().unwrap().1 == false,
                    "{}{}",
                    p_reg,
                    holder
                );
            }
            for (rentor, borrowed) in to_relase.iter() {
                holders.remove(borrowed);
                rentors.remove(rentor);
            }

            index += 1;
        }

        bb.as_mut().insts = new_insts;
        // unimplemented!()
    }
}

///处理因为handle spill产生的多余指令
impl Func {
    ///都是需要用到的时候才进行寄存器值得归还与存储
    ///但是在不同块之间的时候可以用寄存器的借还操作代替从内存空间读取值的操作
    pub fn replace_inst_suf_spill(&mut self) {
        //进行寄存器之间的移动操作
        self.calc_live_base();
        //如果只有一个前继块,则前继块中的spilling优先使用可用的物理寄存器移动到后方
        for bb in self.blocks.iter() {
            //统计bb开头的spilling寄存器的恢复操作
            let live_in = bb.live_in.clone();
            let mut availables = RegUsedStat::init_unspecial_regs();
            live_in
                .iter()
                .for_each(|reg| availables.use_reg(reg.get_color()));
            //对于live in中存在的寄存器不能够借用
            //其他寄存器能够用来做中转,记录第一个遇到的时候的指令
            let mut first_load: HashMap<Reg, (ObjPtr<LIRInst>, RegUsedStat)> = HashMap::new();
            let mut regs_used = availables;
            for inst in bb.insts.iter() {
                match inst.get_type() {
                    InstrsType::LoadFromStack => {
                        let dst_reg = inst.get_def_reg().unwrap();
                        if !first_load.contains_key(dst_reg)
                            && regs_used.is_available_reg(dst_reg.get_color())
                        {
                            first_load.insert(*dst_reg, (*inst, regs_used));
                        }
                    }
                    _ => (),
                }
                for reg in inst.get_regs() {
                    regs_used.use_reg(reg.get_color());
                }
            }

            //通过regs_used传递已经使用到的寄存器
            let mut regs_used = availables;
            //记录下了所有需要load的情况后,查看前文,寻找前文store的情况,判断是否能替换
            for (reg, (inst, reg_use_stat)) in first_load.iter_mut() {
                reg_use_stat.merge(&regs_used);
                //判断是否是可以用来处理的spilling的寄存器情况
                let mut if_ok = false;
                let mut in_cases: HashMap<ObjPtr<BB>, (usize, RegUsedStat)> = HashMap::new();
                for in_bb in bb.in_edge.iter() {
                    debug_assert!(!in_bb.live_out.contains(reg));
                    let mut available_reg_for_in_bb = RegUsedStat::new();
                    for inst in in_bb.insts.iter() {}
                }
                if !if_ok {
                    continue;
                }
            }
        }
    }
}
