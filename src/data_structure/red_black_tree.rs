use core::fmt;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::option::Option::Some;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Color {
    Red,
    Black,
}

#[derive(Debug)]
struct Node {
    key: i32,
    parent: Option<Weak<RefCell<Node>>>,
    left: Option<Rc<RefCell<Node>>>,
    right: Option<Rc<RefCell<Node>>>,
    color: Color,
}

pub struct RedBlackTree {
    root: Option<Rc<RefCell<Node>>>,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum InsertSituation {
    LL,
    LR,
    RL,
    RR,
    Recursion,
    Stable,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum DeleteSituation {
    RLRR,
    RLRE,
    RLER,
    RLEE,
    RRRR,
    RRER,
    RRRE,
    RREE,
    BLR,
    BLBRW,
    BLBER,
    BLBEE,
    BRR,
    BRBWR,
    BRBRE,
    BRBEE,
    Stable,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum DeleteRecursionSituation {
    LRBW,
    LRRB,
    LRRR,
    LBBBB,
    LBBWR,
    LBBRB,
    LBR,
    RRWB,
    RRBR,
    RRRR,
    RBBBB,
    RBBRW,
    RBBBR,
    RBR,
    Stable,
}

///
impl RedBlackTree {
    pub fn new() -> Self {
        RedBlackTree { root: None }
    }

    pub fn insert(&mut self, key: i32) {
        let node_rc = Rc::new(RefCell::new(Node {
            key,
            parent: None,
            left: None,
            right: None,
            color: Color::Red,
        }));
        match &self.root {
            None => {
                node_rc.borrow_mut().color = Color::Black;
                self.root = Some(node_rc);
            }
            Some(root) => {
                let mut parent_rc = Rc::clone(root);
                let son_rc = Rc::clone(&node_rc);
                loop {
                    //借用时候不能修改变量指向
                    //加括号层级是为了限定parent_rc的可变借用范围，从而实现借用修改分离
                    //cur_rc是括号外层级的变量，可以记住内层级的修改
                    let cur_rc;
                    {
                        let mut parent = parent_rc.borrow_mut();
                        cur_rc = if key < parent.key {
                            match &parent.left {
                                Some(son_ref) => {
                                    Rc::clone(son_ref)
                                }
                                None => {
                                    //插入新节点
                                    node_rc.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));
                                    parent.left = Some(node_rc);
                                    break;
                                }
                            }
                        } else if key > parent.key {
                            match &parent.right {
                                Some(son_ref) => {
                                    Rc::clone(son_ref)
                                }
                                None => {
                                    //插入新节点
                                    node_rc.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));
                                    parent.right = Some(node_rc);
                                    break;
                                }
                            }
                        } else {
                            //相等情况暂不处理
                            return;
                        }
                    }
                    //借用结束再修改父节点
                    parent_rc = Rc::clone(&cur_rc);
                }
                self.insert_balance(&parent_rc, &son_rc)
            }
        }
    }

    /// 删除节点，脱离树，树节点不再指向删除节点
    /// 调整删除节点位置和颜色,使情况简单化
    /// 删除节点有三种情况
    /// 1.删除节点没有子节点
    /// 2.删除节点只有一个子节点，且必定为红色
    /// 3.删除节点有两个子节点
    /// 通过转换，全部转换为情况一，删除节点转换为删除叶子节点
    pub fn delete(&mut self, key: i32) {
        //找到删除节点
        let target_option = Self::find(&self.root, key);
        if let Some(target_ref) = &target_option {
            //为了提前释放target的借用
            let mut target_parent_option = None;
            let mut target_left_option = None;
            let mut target_right_option = None;
            let mut target_color = Color::Red;
            {
                let target = target_ref.borrow();
                if let Some(target_parent_weak) = &target.parent {
                    if let Some(target_parent_ref) = &target_parent_weak.upgrade() {
                        target_parent_option = Some(Rc::clone(target_parent_ref));
                    }
                }
                if let Some(target_left_ref) = &target.left {
                    target_left_option = Some(Rc::clone(target_left_ref));
                }
                if let Some(target_right_ref) = &target.right {
                    target_right_option = Some(Rc::clone(target_right_ref));
                }
                target_color = target.color;
            }
            match (&target_left_option, &target_right_option) {
                //1.删除节点没有子节点
                // 处理根关系，斩断连接，删除黑色节点需要平衡
                (None, None) => {
                    match &target_parent_option {
                        None => {
                            self.root = None;
                        }
                        Some(parent_ref) => {
                            {
                                let mut parent = parent_ref.borrow_mut();
                                if let Some(parent_left_ref) = &parent.left {
                                    if Rc::ptr_eq(parent_left_ref, target_ref) {
                                        parent.left = None;
                                    }
                                }
                                if let Some(parent_right_ref) = &parent.right {
                                    if Rc::ptr_eq(parent_right_ref, target_ref) {
                                        parent.right = None;
                                    }
                                }
                            }
                            //删除黑色节点需要调平
                            if target_color == Color::Black {
                                self.delete_balance(parent_ref);
                            }
                        }
                    }
                }
                //2.删除节点只有一个子节点，则删除节点必为黑色，其子节点且必定为红色
                // 李代桃僵，红色子节点代为离去即可，无需平衡
                (Some(son_ref), None) | (None, Some(son_ref)) => {
                    let mut son = son_ref.borrow_mut();
                    son.color = Color::Black;
                    match &target_parent_option {
                        None => {
                            self.root = Some(Rc::clone(son_ref));
                        }
                        Some(parent_ref) => {
                            let mut parent = parent_ref.borrow_mut();
                            son.parent = Some(Rc::downgrade(parent_ref));
                            if let Some(parent_left_ref) = &parent.left {
                                if Rc::ptr_eq(parent_left_ref, target_ref) {
                                    parent.left = Some(Rc::clone(son_ref));
                                }
                            }
                            if let Some(parent_right_ref) = &parent.right {
                                if Rc::ptr_eq(parent_right_ref, target_ref) {
                                    parent.right = Some(Rc::clone(son_ref));
                                }
                            }
                        }
                    }
                }
                //3.删除节点有两个子节点
                // 右子树寻找后继节点，改为删除后继节点
                // 后继节点如有子节点，则必为红色右子节点，李代桃僵即可，后继节点为黑色且没有子节点，需要平衡
                (Some(target_left_ref), Some(target_right_ref)) => {
                    //寻找后继节点
                    let successor_rc = Self::find_minimum(target_right_ref);
                    let successor_ref = &successor_rc;
                    let mut need_balance = false;
                    let mut successor_parent_rc = Rc::clone(successor_ref);
                    let mut successor_right_option = None;
                    {
                        let successor = successor_rc.borrow();
                        if let Some(successor_parent_weak) = &successor.parent {
                            if let Some(successor_parent_ref) = &successor_parent_weak.upgrade() {
                                successor_parent_rc = Rc::clone(successor_parent_ref);
                            }
                        }
                        if let Some(successor_right_ref) = &successor.right {
                            successor_right_option = Some(Rc::clone(successor_right_ref));
                        }
                    }
                    {
                        //后继节点取代删除节点(左连接)
                        successor_rc.borrow_mut().left = Some(Rc::clone(target_left_ref));
                        target_left_ref.borrow_mut().parent = Some(Rc::downgrade(successor_ref));
                    }
                    {
                        //后继节点取代删除节点(右连接)
                        //后继节点必有父节点
                        let mut successor = successor_ref.borrow_mut();
                        let mut successor_parent = successor_parent_rc.borrow_mut();
                        //如果后继节点有右子节点
                        if let Some(successor_right_ref) = &successor_right_option {
                            let mut successor_right = successor_right_ref.borrow_mut();
                            //删除节点的右节点不是后继节点
                            if !Rc::ptr_eq(successor_ref, target_right_ref) {
                                //后继节点的右子节点补位
                                successor_parent.left = successor.right.take();
                                successor_right.parent = Some(Rc::downgrade(&successor_parent_rc));
                                //后继节点取代删除节点(右连接)
                                successor.right = Some(Rc::clone(target_right_ref));
                                //处理特殊情况的借用,处理多次借用问题,删除节点的右子节点就是后继节点的父节点
                                if Rc::ptr_eq(target_right_ref, &successor_parent_rc) {
                                    successor_parent.parent = Some(Rc::downgrade(successor_ref));
                                } else {
                                    target_right_ref.borrow_mut().parent = Some(Rc::downgrade(successor_ref));
                                }
                            }
                            successor_right.color = successor.color;
                        } else {
                            //删除节点的右节点不是后继节点
                            if !Rc::ptr_eq(successor_ref, target_right_ref) {
                                //后继节点的父节点的左节点置空
                                successor_parent.left = None;
                                //后继节点取代删除节点(右连接)
                                successor.right = Some(Rc::clone(target_right_ref));
                                if Rc::ptr_eq(target_right_ref, &successor_parent_rc) {
                                    //处理特殊情况的借用,处理多次借用问题,删除节点的右子节点就是后继节点的父节点
                                    successor_parent.parent = Some(Rc::downgrade(successor_ref));
                                } else {
                                    target_right_ref.borrow_mut().parent = Some(Rc::downgrade(successor_ref));
                                }
                            }
                            //删除黑色节点需要调平
                            if successor.color == Color::Black {
                                need_balance = true;
                            }
                        }
                    }
                    //后继节点取代删除节点(上连接)
                    match &target_parent_option {
                        None => {
                            self.root = Some(Rc::clone(successor_ref));
                            successor_ref.borrow_mut().parent = None;
                        }
                        Some(parent_ref) => {
                            let mut parent = parent_ref.borrow_mut();
                            successor_rc.borrow_mut().parent = Some(Rc::downgrade(parent_ref));
                            if let Some(parent_left_ref) = &parent.left {
                                if Rc::ptr_eq(parent_left_ref, target_ref) {
                                    parent.left = Some(Rc::clone(successor_ref));
                                }
                            }
                            if let Some(parent_right_ref) = &parent.right {
                                if Rc::ptr_eq(parent_right_ref, target_ref) {
                                    parent.right = Some(Rc::clone(successor_ref));
                                }
                            }
                        }
                    }
                    {
                        //后继节点取代删除节点(颜色)
                        successor_rc.borrow_mut().color = target_color;
                    }
                    //需要调平
                    if need_balance {
                        if Rc::ptr_eq(&successor_parent_rc, target_ref) {
                            //处理后继节点的父节点就是目标节点的特殊情况，删除平衡方法需要传入删除节点的现父节点
                            //多数情况下，后继节点的父节点就是删除节点的现父节点
                            //只有目标节点的右节点就是后继节点且后继节点没有子节点的情况下，才会出现后继节点取代后继节点父节点的情况
                            self.delete_balance(successor_ref);
                        } else {
                            self.delete_balance(&successor_parent_rc);
                        }
                    }
                }
            }
        }
    }

    pub fn get(&self, key: i32) -> Option<i32> {
        match &self.root {
            None => {
                return None;
            }
            Some(root_ref) => {
                let mut next_rc = Rc::clone(root_ref);
                let mut cur_rc;
                loop {
                    cur_rc = next_rc;
                    let cur_ref = &cur_rc;
                    let cur = cur_ref.borrow();
                    match key.cmp(&cur.key) {
                        std::cmp::Ordering::Equal => {
                            return Some(cur.key);
                        }
                        std::cmp::Ordering::Less => {
                            match &cur.left {
                                None => { return None; }
                                Some(left_ref) => {
                                    next_rc = Rc::clone(left_ref);
                                }
                            }
                        }
                        std::cmp::Ordering::Greater => {
                            match &cur.right {
                                None => { return None; }
                                Some(right_ref) => {
                                    next_rc = Rc::clone(right_ref);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        Self::count_size(&self.root)
    }

    pub fn preorder_traversal(&self) {
        println!("preorder_traversal");
        if let Some(root) = &self.root {
            Self::do_preorder_traversal(&*root.as_ref().borrow())
        }
    }

    pub fn inorder_traversal(&self) {
        println!("inorder_traversal");
        if let Some(root) = &self.root {
            Self::do_inorder_traversal(&*root.as_ref().borrow())
        }
    }

    pub fn postorder_traversal(&self) {
        println!("postorder_traversal");
        if let Some(root) = &self.root {
            Self::do_postorder_traversal(&*root.as_ref().borrow())
        }
    }

    ///左旋
    fn rotate_left(&mut self, grand_parent_ref: &Rc<RefCell<Node>>, parent_ref: &Rc<RefCell<Node>>) {
        let mut parent = parent_ref.borrow_mut();
        let mut grand_parent = grand_parent_ref.borrow_mut();
        if let Some(brother_ref) = &parent.left {
            brother_ref.borrow_mut().parent = Some(Rc::downgrade(grand_parent_ref));
            grand_parent.right = Some(Rc::clone(brother_ref));
        } else {
            grand_parent.right = None;
        }
        if let Some(grand_parent_parent_weak) = &grand_parent.parent {
            if let Some(grand_parent_parent_ref) = &grand_parent_parent_weak.upgrade() {
                let mut grand_parent_parent = grand_parent_parent_ref.borrow_mut();
                parent.parent = Some(Rc::downgrade(grand_parent_parent_ref));
                if let Some(grand_parent_parent_left_ref) = &grand_parent_parent.left {
                    if Rc::ptr_eq(grand_parent_parent_left_ref, grand_parent_ref) {
                        grand_parent_parent.left = Some(Rc::clone(parent_ref));
                    }
                }
                if let Some(grand_parent_parent_right_ref) = &grand_parent_parent.right {
                    if Rc::ptr_eq(grand_parent_parent_right_ref, grand_parent_ref) {
                        grand_parent_parent.right = Some(Rc::clone(parent_ref));
                    }
                }
            }
        } else {
            parent.parent = None;
            self.root = Some(Rc::clone(parent_ref));
        }
        parent.left = Some(Rc::clone(grand_parent_ref));
        grand_parent.parent = Some(Rc::downgrade(parent_ref));
    }

    ///右旋
    fn rotate_right(&mut self, grand_parent_ref: &Rc<RefCell<Node>>, parent_ref: &Rc<RefCell<Node>>) {
        let mut parent = parent_ref.borrow_mut();
        let mut grand_parent = grand_parent_ref.borrow_mut();
        if let Some(brother_ref) = &parent.right {
            brother_ref.borrow_mut().parent = Some(Rc::downgrade(grand_parent_ref));
            grand_parent.left = Some(Rc::clone(brother_ref));
        } else {
            grand_parent.left = None;
        }
        if let Some(grand_parent_parent_weak) = &grand_parent.parent {
            if let Some(grand_parent_parent_ref) = &grand_parent_parent_weak.upgrade() {
                let mut grand_parent_parent = grand_parent_parent_ref.borrow_mut();
                parent.parent = Some(Rc::downgrade(grand_parent_parent_ref));
                if let Some(grand_parent_parent_left_ref) = &grand_parent_parent.left {
                    if Rc::ptr_eq(grand_parent_parent_left_ref, grand_parent_ref) {
                        grand_parent_parent.left = Some(Rc::clone(parent_ref));
                    }
                }
                if let Some(grand_parent_parent_right_ref) = &grand_parent_parent.right {
                    if Rc::ptr_eq(grand_parent_parent_right_ref, grand_parent_ref) {
                        grand_parent_parent.right = Some(Rc::clone(parent_ref));
                    }
                }
            }
        } else {
            parent.parent = None;
            self.root = Some(Rc::clone(parent_ref));
        }
        parent.right = Some(Rc::clone(grand_parent_ref));
        grand_parent.parent = Some(Rc::downgrade(parent_ref));
    }

    /// 插入平衡
    /// 1.父节点为黑色，不需要平衡操作
    /// 2.父节点为红色
    /// 2.1.叔节点为黑色或者不存在
    /// 存在LL,LR,RL,RR的情况
    /// 2.2.叔节点为红色 上溢情况
    /// 需要把父节点和叔节点染黑，爷节点染红，以爷节点为新插入的节点，递归平衡操作
    fn insert_balance(&mut self, parent_ref: &Rc<RefCell<Node>>, son_ref: &Rc<RefCell<Node>>) {
        let (insert_situation, grand_parent_rc, uncle_rc) = Self::judge_insert_situation(parent_ref, son_ref);
        match insert_situation {
            InsertSituation::LL => {
                self.rotate_right(&grand_parent_rc, parent_ref);
                grand_parent_rc.borrow_mut().color = Color::Red;
                parent_ref.borrow_mut().color = Color::Black;
            }
            InsertSituation::RR => {
                self.rotate_left(&grand_parent_rc, parent_ref);
                grand_parent_rc.borrow_mut().color = Color::Red;
                parent_ref.borrow_mut().color = Color::Black;
            }
            InsertSituation::LR => {
                self.rotate_left(parent_ref, son_ref);
                self.rotate_right(&grand_parent_rc, son_ref);
                grand_parent_rc.borrow_mut().color = Color::Red;
                son_ref.borrow_mut().color = Color::Black;
            }
            InsertSituation::RL => {
                self.rotate_right(parent_ref, son_ref);
                self.rotate_left(&grand_parent_rc, son_ref);
                grand_parent_rc.borrow_mut().color = Color::Red;
                son_ref.borrow_mut().color = Color::Black;
            }
            InsertSituation::Recursion => {
                let mut grand_parent_parent_rc = Rc::clone(&grand_parent_rc);
                let mut grand_parent_color = Color::Red;
                //缩小借用范围
                {
                    parent_ref.borrow_mut().color = Color::Black;
                    uncle_rc.borrow_mut().color = Color::Black;
                    match &grand_parent_rc.borrow().parent {
                        Some(grand_parent_parent_weak) => {
                            if let Some(grand_parent_parent_ref) = &grand_parent_parent_weak.upgrade() {
                                grand_parent_parent_rc = Rc::clone(grand_parent_parent_ref);
                            }
                        }
                        //爷节点已经是根节点，结束递归
                        None => {
                            grand_parent_color = Color::Black;
                        }
                    }
                    grand_parent_rc.borrow_mut().color = grand_parent_color;
                }
                match grand_parent_color {
                    Color::Red => {
                        //以爷节点为新插入的节点，递归平衡操作
                        self.insert_balance(&grand_parent_parent_rc, &grand_parent_rc);
                    }
                    Color::Black => {
                        //爷节点已经是根节点，结束递归
                        self.root = Some(Rc::clone(&grand_parent_rc));
                    }
                }
            }
            InsertSituation::Stable => {}
        }
    }

    ///删除平衡
    /// parent_ref为已删除节点的父节点
    /// 删除节点已脱离树，不再有树节点指向删除节点
    /// 以删除节点为左子节点为例,有以下情况
    ///1.父节点是红色的
    /// 兄弟节点一定为黑色,其子节点存在则必为红色
    /// 1.1兄弟节点有两个子节点
    /// 1.2兄弟节点只有一个左子节点
    /// 1.3兄弟节点只有一个右子节点
    /// 1.4兄弟节点没有子节点
    ///2.父节点是黑色的
    /// 2.1兄弟节点为红色，兄弟节点必定有两个黑子(且黑子可能有子节点，如有则必为红色)
    /// 2.2兄弟节点为黑色，有两个子节点 或 只有个左子节点(必定为红色)
    /// 2.3兄弟节点为黑色，且只有一个右子节点
    /// 2.4兄弟节点为黑色，且没有子节点
    ///删除节点为右节点时，对称以上情况即可
    fn delete_balance(&mut self, parent_ref: &Rc<RefCell<Node>>) {
        let (situation, brother_rc, brother_left_rc, brother_right_rc) = Self::judge_delete_situation(parent_ref);
        match situation {
            //1.父节点是红色的
            //兄弟节点一定为黑色,其子节点存在则必为红色
            //删除节点是左节点，兄弟节点为右节点
            //1.1兄弟节点有两个子节点，且必为红色
            DeleteSituation::RLRR => {
                self.rotate_left(parent_ref, &brother_rc);
                brother_rc.borrow_mut().color = Color::Red;
                parent_ref.borrow_mut().color = Color::Black;
                brother_right_rc.borrow_mut().color = Color::Black;
            }
            //1.2兄弟节点只有一个左子节点，且必为红色
            DeleteSituation::RLRE => {
                self.rotate_right(&brother_rc, &brother_left_rc);
                self.rotate_left(parent_ref, &brother_left_rc);
                parent_ref.borrow_mut().color = Color::Black;
            }
            //1.3兄弟节点只有一个右子节点，且必为红色
            DeleteSituation::RLER => {
                self.rotate_left(parent_ref, &brother_rc);
            }
            //1.4兄弟节点没有子节点
            DeleteSituation::RLEE => {
                parent_ref.borrow_mut().color = Color::Black;
                brother_rc.borrow_mut().color = Color::Red;
            }
            //删除节点是右节点,，兄弟节点为左节点
            //1.1兄弟节点有两个子节点，且必为红色
            DeleteSituation::RRRR => {
                self.rotate_right(parent_ref, &brother_rc);
                brother_rc.borrow_mut().color = Color::Red;
                parent_ref.borrow_mut().color = Color::Black;
                brother_left_rc.borrow_mut().color = Color::Black;
            }
            //1.2兄弟节点只有一个右子节点，且必为红色
            DeleteSituation::RRER => {
                self.rotate_left(&brother_rc, &brother_right_rc);
                self.rotate_right(parent_ref, &brother_right_rc);
                parent_ref.borrow_mut().color = Color::Black;
            }
            //1.3兄弟节点只有一个左子节点，且必为红色
            DeleteSituation::RRRE => {
                self.rotate_right(parent_ref, &brother_rc);
            }
            //1.4没有侄子节点
            DeleteSituation::RREE => {
                parent_ref.borrow_mut().color = Color::Black;
                brother_rc.borrow_mut().color = Color::Red;
            }
            //2.父节点是黑色的
            //兄弟节点一定存在
            //删除节点是左节点,兄弟节点为右节点
            //2.1兄弟节点为红色
            //兄弟节点必定有两个子节点，且为黑色
            DeleteSituation::BLR => {
                self.rotate_left(parent_ref, &brother_rc);
                self.rotate_left(parent_ref, &brother_left_rc);
                {
                    brother_rc.borrow_mut().color = Color::Black;
                    parent_ref.borrow_mut().color = Color::Red;
                }
                //如果旋转前，兄弟节点的左子节点存在左子节点，
                //即原来的父节点，旋转过后存在右子节点，则需要对其做插入调平处理
                let mut parent_now_right_option = None;
                {
                    if let Some(parent_now_right_ref) = &parent_ref.borrow().right {
                        parent_now_right_option = Some(Rc::clone(parent_now_right_ref));
                    }
                }
                if let Some(parent_now_right_ref) = &parent_now_right_option {
                    self.insert_balance(parent_ref, parent_now_right_ref);
                }
            }
            //兄弟节点为黑色
            //2.2兄弟节点为黑色，有两个子节点 或 只有个左子节点，子节点一定是红色的
            DeleteSituation::BLBRW => {
                self.rotate_right(&brother_rc, &brother_left_rc);
                self.rotate_left(parent_ref, &brother_left_rc);
                brother_left_rc.borrow_mut().color = Color::Black;
            }
            //2.3兄弟节点为黑色，且只有一个右子节点
            DeleteSituation::BLBER => {
                self.rotate_left(parent_ref, &brother_rc);
                brother_right_rc.borrow_mut().color = Color::Black;
            }
            //2.4兄弟节点为黑色，且没有子节点
            DeleteSituation::BLBEE => {
                //先达到局部平衡
                {
                    brother_rc.borrow_mut().color = Color::Red;
                }
                self.delete_balance_recursion(parent_ref);
            }
            //删除节点是右节点,兄弟节点为左节点
            //2.1兄弟节点为红色
            //兄弟节点必定有两个子节点，且为黑色
            DeleteSituation::BRR => {
                self.rotate_right(parent_ref, &brother_rc);
                self.rotate_right(parent_ref, &brother_right_rc);
                {
                    brother_rc.borrow_mut().color = Color::Black;
                    parent_ref.borrow_mut().color = Color::Red;
                }
                //如果旋转前，兄弟节点的右子节点存在右子节点，
                //即原来的父节点，旋转过后存在左子节点，则需要对其做插入调平处理
                let mut parent_now_left_option = None;
                {
                    if let Some(parent_now_left_ref) = &parent_ref.borrow().left {
                        parent_now_left_option = Some(Rc::clone(parent_now_left_ref));
                    }
                }
                if let Some(parent_now_left_ref) = &parent_now_left_option {
                    self.insert_balance(parent_ref, parent_now_left_ref);
                }
            }
            //兄弟节点为黑色
            //2.2兄弟节点为黑色，有两个子节点 或 只有个右子节点，子节点一定是红色的
            DeleteSituation::BRBWR => {
                self.rotate_left(&brother_rc, &brother_right_rc);
                self.rotate_right(parent_ref, &brother_right_rc);
                brother_right_rc.borrow_mut().color = Color::Black;
            }
            //2.3兄弟节点为黑色，且只有一个左子节点
            DeleteSituation::BRBRE => {
                self.rotate_right(parent_ref, &brother_rc);
                brother_left_rc.borrow_mut().color = Color::Black;
            }
            //2.4兄弟节点为黑色，且没有子节点
            DeleteSituation::BRBEE => {
                //先达到局部平衡
                {
                    brother_rc.borrow_mut().color = Color::Red;
                }
                self.delete_balance_recursion(parent_ref);
            }
            DeleteSituation::Stable => {}
        }
    }

    /// 处理删除平衡操作的失衡情况
    /// target_ref为失衡节点
    /// 失衡节点为局部平衡后的根节点
    fn delete_balance_recursion(&mut self, target_ref: &Rc<RefCell<Node>>) {
        let (situation, parent_rc, brother_rc, brother_left_rc, brother_right_rc) = Self::judge_delete_recursion_situation(target_ref);
        match situation {
            //失衡节点为左节点
            //1.父节点是红色
            //1.1兄弟的左子节点为黑色
            DeleteRecursionSituation::LRBW => {
                self.rotate_left(&parent_rc, &brother_rc);
            }
            //1.2兄弟的左子节点为红色，兄弟的右子节点为黑色
            DeleteRecursionSituation::LRRB => {
                {
                    parent_rc.borrow_mut().color = Color::Black;
                    brother_rc.borrow_mut().color = Color::Red;
                }
                self.insert_balance(&brother_rc, &brother_left_rc);
            }
            //1.3兄弟的两个子节点都为红色
            DeleteRecursionSituation::LRRR => {
                {
                    parent_rc.borrow_mut().color = Color::Black;
                    brother_rc.borrow_mut().color = Color::Red;
                    brother_right_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_left(&parent_rc, &brother_rc);
            }
            //2.父节点是黑色
            //兄弟节点为黑色
            //2.1兄弟的两子节点都为黑色
            DeleteRecursionSituation::LBBBB => {
                {
                    brother_rc.borrow_mut().color = Color::Red;
                }
                //继续求助上级
                self.delete_balance_recursion(&parent_rc);
            }
            //2.2兄弟的右子节点为红色
            DeleteRecursionSituation::LBBWR => {
                self.rotate_left(&parent_rc, &brother_rc);
                brother_right_rc.borrow_mut().color = Color::Black;
            }
            DeleteRecursionSituation::LBBRB => {
                {
                    brother_left_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_right(&brother_rc, &brother_left_rc);
                self.rotate_left(&parent_rc, &brother_left_rc);
            }
            DeleteRecursionSituation::LBR => {
                {
                    parent_rc.borrow_mut().color = Color::Red;
                    brother_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_left(&parent_rc, &brother_rc);
                //转为情况1，继续递归
                self.delete_balance_recursion(target_ref);
            }
            //失衡节点为右节点
            //1.父节点是红色
            //1.1兄弟的右子节点为黑色
            DeleteRecursionSituation::RRWB => {
                self.rotate_right(&parent_rc, &brother_rc);
            }
            //1.2兄弟的右子节点为红色，兄弟的左子节点为黑色
            DeleteRecursionSituation::RRBR => {
                {
                    parent_rc.borrow_mut().color = Color::Black;
                    brother_rc.borrow_mut().color = Color::Red;
                }
                self.insert_balance(&brother_rc, &brother_right_rc);
            }
            //1.3兄弟的两个子节点都为红色
            DeleteRecursionSituation::RRRR => {
                {
                    parent_rc.borrow_mut().color = Color::Black;
                    brother_rc.borrow_mut().color = Color::Red;
                    brother_left_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_right(&parent_rc, &brother_rc);
            }
            //2.父节点是黑色
            //兄弟节点为黑色
            //2.1兄弟的两子节点都为黑色
            DeleteRecursionSituation::RBBBB => {
                {
                    brother_rc.borrow_mut().color = Color::Red;
                }
                //继续求助上级
                self.delete_balance_recursion(&parent_rc);
            }
            //2.2兄弟的左子节点为红色
            DeleteRecursionSituation::RBBRW => {
                self.rotate_right(&parent_rc, &brother_rc);
                brother_left_rc.borrow_mut().color = Color::Black;
            }
            //2.3兄弟的右子节点为红色，兄弟的左子节点为黑色
            DeleteRecursionSituation::RBBBR => {
                {
                    brother_right_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_left(&brother_rc, &brother_right_rc);
                self.rotate_right(&parent_rc, &brother_right_rc);
            }
            //兄弟节点为红色
            //2.4兄弟节点为红色
            DeleteRecursionSituation::RBR => {
                {
                    parent_rc.borrow_mut().color = Color::Red;
                    brother_rc.borrow_mut().color = Color::Black;
                }
                self.rotate_right(&parent_rc, &brother_rc);
                //转为情况1，继续递归
                self.delete_balance_recursion(target_ref);
            }
            DeleteRecursionSituation::Stable => {}
        }
    }

    ///寻找最小节点
    fn find_minimum(node_ref: &Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
        let mut next_rc = Rc::clone(&node_ref);
        loop {
            let cur_rc = Rc::clone(&next_rc);
            let cur = cur_rc.borrow();
            match &cur.left {
                Some(next_ref) => {
                    next_rc = Rc::clone(next_ref);
                }
                None => {
                    return next_rc;
                }
            }
        }
    }

    fn find(cur_option: &Option<Rc<RefCell<Node>>>, key: i32) -> Option<Rc<RefCell<Node>>> {
        match cur_option {
            Some(cur_ref) => {
                let cur = cur_ref.borrow();
                return match key.cmp(&cur.key) {
                    std::cmp::Ordering::Equal => {
                        Some(Rc::clone(cur_ref))
                    }
                    std::cmp::Ordering::Less => {
                        Self::find(&cur.left, key)
                    }
                    std::cmp::Ordering::Greater => {
                        Self::find(&cur.right, key)
                    }
                };
            }
            None => {
                None
            }
        }
    }

    fn judge_insert_situation(parent_ref: &Rc<RefCell<Node>>, son_ref: &Rc<RefCell<Node>>) -> (InsertSituation, Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
        let mut insert_situation = InsertSituation::Stable;
        let mut grand_parent_rc = Rc::clone(parent_ref);
        let mut uncle_rc = Rc::clone(parent_ref);
        let parent = parent_ref.borrow();
        //父节点为黑色不需要平衡
        //只需要处理父节点为红色的情况
        if parent.color == Color::Red {
            //出现父子连续红，爷节点一定存在
            if let Some(grand_parent_weak) = &parent.parent {
                if let Some(grand_parent_ref) = &grand_parent_weak.upgrade() {
                    grand_parent_rc = Rc::clone(grand_parent_ref);
                    let grand_parent = grand_parent_ref.borrow();
                    //父节点是左节点
                    if let Some(grand_parent_left_ref) = &grand_parent.left {
                        if Rc::ptr_eq(grand_parent_left_ref, parent_ref) {
                            //2.2.上溢情况，叔节点存在，并且为红色
                            if let Some(grand_parent_right_ref) = &grand_parent.right {
                                uncle_rc = Rc::clone(grand_parent_right_ref);
                                let uncle = grand_parent_right_ref.borrow();
                                if uncle.color == Color::Red {
                                    insert_situation = InsertSituation::Recursion;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                            //2.1.叔节点不存在，或者为黑色
                            //LL
                            if let Some(parent_left_ref) = &parent.left {
                                if Rc::ptr_eq(parent_left_ref, son_ref) {
                                    insert_situation = InsertSituation::LL;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                            //LR
                            if let Some(parent_right_ref) = &parent.right {
                                if Rc::ptr_eq(parent_right_ref, son_ref) {
                                    insert_situation = InsertSituation::LR;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                        }
                    }
                    //父节点是右节点
                    if let Some(grand_parent_right_ref) = &grand_parent.right {
                        if Rc::ptr_eq(grand_parent_right_ref, parent_ref) {
                            //2.2.上溢情况，叔节点存在，并且为红色
                            if let Some(grand_parent_left_ref) = &grand_parent.left {
                                uncle_rc = Rc::clone(grand_parent_left_ref);
                                let uncle = grand_parent_left_ref.borrow();
                                if uncle.color == Color::Red {
                                    insert_situation = InsertSituation::Recursion;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                            //2.1.叔节点不存在，或者为黑色
                            //RR
                            if let Some(parent_right_ref) = &parent.right {
                                if Rc::ptr_eq(parent_right_ref, son_ref) {
                                    insert_situation = InsertSituation::RR;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                            //RL
                            if let Some(parent_left_ref) = &parent.left {
                                if Rc::ptr_eq(parent_left_ref, son_ref) {
                                    insert_situation = InsertSituation::RL;
                                    return (insert_situation, grand_parent_rc, uncle_rc);
                                }
                            }
                        }
                    }
                }
            }
        }
        (insert_situation, grand_parent_rc, uncle_rc)
    }

    fn judge_delete_situation(parent_ref: &Rc<RefCell<Node>>) -> (DeleteSituation, Rc<RefCell<Node>>, Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
        let parent = parent_ref.borrow();
        return match parent.color {
            //1.父节点是红色的
            Color::Red => {
                //兄弟节点一定为黑色,其子节点存在则必为红色
                //删除节点是左节点，兄弟节点为右节点
                if let Some(brother_ref) = &parent.right {
                    let brother = brother_ref.borrow();
                    return match (&brother.left, &brother.right) {
                        //1.1兄弟节点有两个子节点，且必为红色
                        (Some(_), Some(brother_right_ref)) => {
                            (DeleteSituation::RLRR, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(brother_right_ref))
                        }
                        //1.2兄弟节点只有一个左子节点，且必为红色
                        (Some(brother_left_ref), None) => {
                            (DeleteSituation::RLRE, Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(parent_ref))
                        }
                        //1.3兄弟节点只有一个右子节点，且必为红色
                        (None, Some(_)) => {
                            (DeleteSituation::RLER, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                        }
                        //1.4兄弟节点没有子节点
                        (None, None) => {
                            (DeleteSituation::RLEE, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                        }
                    };
                }
                //删除节点是右节点,，兄弟节点为左节点
                if let Some(brother_ref) = &parent.left {
                    let brother = brother_ref.borrow();
                    return match (&brother.left, &brother.right) {
                        //1.1兄弟节点有两个子节点，且必为红色
                        (Some(brother_left_ref), Some(_)) => {
                            (DeleteSituation::RRRR, Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(parent_ref))
                        }
                        //1.2兄弟节点只有一个右子节点，且为红色
                        (None, Some(brother_right_ref)) => {
                            (DeleteSituation::RRER, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(brother_right_ref))
                        }
                        //1.3兄弟节点只有一个左子节点，且为红色
                        (Some(_), None) => {
                            (DeleteSituation::RRRE, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                        }
                        //1.4兄弟节点没有子节点
                        (None, None) => {
                            (DeleteSituation::RREE, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                        }
                    };
                }
                (DeleteSituation::RLER, Rc::clone(parent_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
            }
            //2.父节点是黑色的
            Color::Black => {
                //兄弟节点一定存在
                //删除节点是左节点,兄弟节点为右节点
                if let Some(brother_ref) = &parent.right {
                    let brother = brother_ref.borrow();
                    match brother.color {
                        //2.1兄弟节点为红色
                        Color::Red => {
                            //兄弟节点必定有两个子节点，且为黑色
                            if let (Some(brother_left_ref), Some(_)) = (&brother.left, &brother.right) {
                                return (DeleteSituation::BLR, Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(parent_ref));
                            }
                        }
                        //兄弟节点为黑色
                        Color::Black => {
                            return match (&brother.left, &brother.right) {
                                //2.2兄弟节点为黑色，有两个子节点 或 只有个左子节点，子节点一定是红色的
                                (Some(brother_left_ref), _) => {
                                    (DeleteSituation::BLBRW, Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(parent_ref))
                                }
                                //2.3兄弟节点为黑色，且只有一个右子节点
                                (None, Some(brother_right_ref)) => {
                                    (DeleteSituation::BLBER, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(brother_right_ref))
                                }
                                //2.4兄弟节点为黑色，且没有子节点
                                (None, None) => {
                                    (DeleteSituation::BLBEE, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                                }
                            }
                        }
                    }
                }
                //删除节点是右节点,兄弟节点为左节点
                if let Some(brother_ref) = &parent.left {
                    let brother = brother_ref.borrow();
                    match brother.color {
                        //2.1兄弟节点为红色
                        Color::Red => {
                            //兄弟节点必定有两个子节点，且为黑色
                            if let (Some(_), Some(brother_right_ref)) = (&brother.left, &brother.right) {
                                return (DeleteSituation::BRR, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(brother_right_ref));
                            }
                        }
                        //兄弟节点为黑色
                        Color::Black => {
                            return match (&brother.left, &brother.right) {
                                //2.2兄弟节点为黑色，有两个子节点 或 只有个右子节点，子节点一定是红色的
                                (_, Some(brother_right_ref)) => {
                                    (DeleteSituation::BRBWR, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(brother_right_ref))
                                }
                                //2.3兄弟节点为黑色，且只有一个左子节点
                                (Some(brother_left_ref), None) => {
                                    (DeleteSituation::BRBRE, Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(parent_ref))
                                }
                                //2.4兄弟节点为黑色，且没有子节点
                                (None, None) => {
                                    (DeleteSituation::BRBEE, Rc::clone(brother_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
                                }
                            }
                        }
                    }
                }
                (DeleteSituation::Stable, Rc::clone(parent_ref), Rc::clone(parent_ref), Rc::clone(parent_ref))
            }
        };
    }

    fn judge_delete_recursion_situation(cur_ref: &Rc<RefCell<Node>>) -> (DeleteRecursionSituation, Rc<RefCell<Node>>, Rc<RefCell<Node>>, Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
        //失衡节点的父节点不存在，即达到了全局平衡
        if let Some(parent_weak) = &cur_ref.borrow().parent {
            if let Some(parent_ref) = &parent_weak.upgrade() {
                let parent = parent_ref.borrow();
                //兄弟节点必定存在,且兄弟两个子节点都必定存在
                if let (Some(parent_left_ref), Some(parent_right_ref)) = (&parent.left, &parent.right) {
                    //失衡节点为左节点
                    if Rc::ptr_eq(parent_left_ref, cur_ref) {
                        let brother_ref = &Rc::clone(parent_right_ref);
                        let brother = brother_ref.borrow();
                        match parent.color {
                            //1.父节点是红色
                            Color::Red => {
                                if let (Some(brother_left_ref), Some(brother_right_ref)) = (&brother.left, &brother.right) {
                                    let brother_left = brother_left_ref.borrow();
                                    let brother_right = brother_right_ref.borrow();
                                    return match (&brother_left.color, &brother_right.color) {
                                        //1.1兄弟的左子节点为黑色
                                        (Color::Black, _) => {
                                            (DeleteRecursionSituation::LRBW, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                        //1.2兄弟的左子节点为红色，兄弟的右子节点为黑色
                                        (Color::Red, Color::Black) => {
                                            (DeleteRecursionSituation::LRRB, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                        //1.3兄弟的两个子节点都为红色
                                        (Color::Red, Color::Red) => {
                                            (DeleteRecursionSituation::LRRR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                    };
                                }
                            }
                            //2.父节点是黑色
                            Color::Black => {
                                if let (Some(brother_left_ref), Some(brother_right_ref)) = (&brother.left, &brother.right) {
                                    let brother_left = brother_left_ref.borrow();
                                    let brother_right = brother_right_ref.borrow();
                                    return match &brother.color {
                                        //兄弟节点为黑色
                                        Color::Black => {
                                            match (&brother_left.color, &brother_right.color) {
                                                //2.1兄弟的两子节点都为黑色
                                                (Color::Black, Color::Black) => {
                                                    (DeleteRecursionSituation::LBBBB, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                                //2.2兄弟的右子节点为红色
                                                (_, Color::Red) => {
                                                    (DeleteRecursionSituation::LBBWR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                                //2.3兄弟的左子节点为红色，兄弟的右子节点为黑色
                                                (Color::Red, Color::Black) => {
                                                    (DeleteRecursionSituation::LBBRB, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                            }
                                        }
                                        //兄弟节点为红色
                                        Color::Red => {
                                            //2.4兄弟节点为红色
                                            (DeleteRecursionSituation::LBR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                    };
                                }
                            }
                        }
                    }
                    //失衡节点为右节点
                    if Rc::ptr_eq(parent_right_ref, cur_ref) {
                        let brother_ref = &Rc::clone(parent_left_ref);
                        let brother = brother_ref.borrow_mut();
                        match parent.color {
                            //1.父节点是红色
                            Color::Red => {
                                if let (Some(brother_left_ref), Some(brother_right_ref)) = (&brother.left, &brother.right) {
                                    let brother_left = brother_left_ref.borrow();
                                    let brother_right = brother_right_ref.borrow();
                                    return match (&brother_left.color, &brother_right.color) {
                                        //1.1兄弟的右子节点为黑色
                                        (_, Color::Black) => {
                                            (DeleteRecursionSituation::RRWB, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                        //1.2兄弟的右子节点为红色，兄弟的左子节点为黑色
                                        (Color::Black, Color::Red) => {
                                            (DeleteRecursionSituation::RRBR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                        //1.3兄弟的两个子节点都为红色
                                        (Color::Red, Color::Red) => {
                                            (DeleteRecursionSituation::RRRR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                    };
                                }
                            }
                            //2.父节点是黑色
                            Color::Black => {
                                if let (Some(brother_left_ref), Some(brother_right_ref)) = (&brother.left, &brother.right) {
                                    let brother_left = brother_left_ref.borrow();
                                    let brother_right = brother_right_ref.borrow();
                                    return match &brother.color {
                                        //兄弟节点为黑色
                                        Color::Black => {
                                            match (&brother_left.color, &brother_right.color) {
                                                //2.1兄弟的两子节点都为黑色
                                                (Color::Black, Color::Black) => {
                                                    (DeleteRecursionSituation::RBBBB, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                                //2.2兄弟的左子节点为红色
                                                (Color::Red, _) => {
                                                    (DeleteRecursionSituation::RBBRW, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                                //2.3兄弟的右子节点为红色，兄弟的左子节点为黑色
                                                (Color::Black, Color::Red) => {
                                                    (DeleteRecursionSituation::RBBBR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                                }
                                            }
                                        }
                                        //兄弟节点为红色
                                        Color::Red => {
                                            //2.4兄弟节点为红色
                                            (DeleteRecursionSituation::RBR, Rc::clone(parent_ref), Rc::clone(brother_ref), Rc::clone(brother_left_ref), Rc::clone(brother_right_ref))
                                        }
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        return (DeleteRecursionSituation::Stable, Rc::clone(cur_ref), Rc::clone(cur_ref), Rc::clone(cur_ref), Rc::clone(cur_ref));
    }

    fn do_preorder_traversal(node: &Node) {
        println!("{}", node);
        if let Some(left) = &node.left {
            Self::do_preorder_traversal(&*left.as_ref().borrow());
        }
        if let Some(right) = &node.right {
            Self::do_preorder_traversal(&*right.as_ref().borrow());
        }
    }

    fn do_inorder_traversal(node: &Node) {
        if let Some(left) = &node.left {
            Self::do_inorder_traversal(&*left.as_ref().borrow());
        }
        println!("{}", node);
        if let Some(right) = &node.right {
            Self::do_inorder_traversal(&*right.as_ref().borrow());
        }
    }

    fn do_postorder_traversal(node: &Node) {
        if let Some(left) = &node.left {
            Self::do_postorder_traversal(&*left.as_ref().borrow());
        }
        if let Some(right) = &node.right {
            Self::do_postorder_traversal(&*right.as_ref().borrow());
        }
        println!("{}", node);
    }

    fn count_size(cur_option: &Option<Rc<RefCell<Node>>>) -> usize {
        return match cur_option {
            Some(cur_ref) => {
                let cur = cur_ref.borrow();
                Self::count_size(&cur.left) + 1 + Self::count_size(&cur.right)
            }
            None => {
                0
            }
        };
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node(Key: {}, Color: {:?}, ",
            self.key, self.color
        )?;

        if let Some(left_ref) = &self.left {
            write!(f, "Left: {}, ", left_ref.borrow().key)?;
        }

        if let Some(right_ref) = &self.right {
            write!(f, "Right: {}, ", right_ref.borrow().key)?;
        }

        if let Some(parent_weak) = &self.parent {
            if let Some(parent_ref) = parent_weak.upgrade() {
                write!(f, "Parent: {}", parent_ref.borrow().key)?;
            }
        }

        write!(f, ")")
    }
}
