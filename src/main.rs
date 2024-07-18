use std::collections::HashMap;
use rand::Rng;
use crate::data_structure::red_black_tree::{RedBlackTree};

pub mod data_structure;

fn main() {
    //红黑树
    let mut rbt = RedBlackTree::new();
    //辅助验证 HashMap
    let mut map = HashMap::new();
    //随机数生成器
    let mut rng = rand::thread_rng();
    //插入次数
    let mut count = 0;
    //插入阶段
    for _ in 0..100_000 {
        let random_number = rng.gen_range(1..=100_000);
        rbt.insert(random_number);
        map.insert(random_number, random_number);
        count += 1;
        // rbt.preorder_traversal();
        println!("size={}==={}==={}==={}", rbt.size(), map.len(), count, random_number);
        if rbt.size() != map.len() {
            println!("插入逻辑出错了");
            return;
        }
    }
    //删除阶段
    while !map.is_empty() {
        let keys: Vec<_> = map.keys().cloned().collect();
        let index_to_delete = rng.gen_range(0..keys.len());
        let key_to_delete = keys[index_to_delete];
        //删除
        rbt.delete(key_to_delete);
        map.remove(&key_to_delete);
        // rbt.preorder_traversal();
        println!("size={}==={}", rbt.size(), map.len());
        if rbt.size() != map.len() {
            println!("删除逻辑出错了");
            return;
        }
    }
}