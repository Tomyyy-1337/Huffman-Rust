use std::{cmp::Reverse, collections::HashMap};

use priority_queue::PriorityQueue;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct HuffmanResult {
    tree: HuffmanTree,
    unused_bits: u8,
    data: Vec<u8>,
}

impl HuffmanResult {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
    
    pub fn deserialize(input: &[u8]) -> Self {
        bincode::deserialize(input).unwrap()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct HuffmanTree {
    children: Vec<HuffmanTree>,
    character: Option<char>,
}

impl HuffmanTree {
    fn build_tree(input: &str) -> Self {
        let chars_counts = input.chars()
            .fold(HashMap::new(), |mut map, c|{
                *map.entry(c).or_insert(0) += 1;
                map
            })
            .into_iter()
            .collect::<Vec<(char, i32)>>();

        let mut pq: PriorityQueue<Self, _, _> = PriorityQueue::new();
        pq.extend(chars_counts.into_iter().map(|(c, count)| (Self {
            children: vec![],
            character: Some(c),
        }, Reverse(count))));

        while pq.len() > 1 {
            let (left, count_left) = pq.pop().unwrap();
            let (right, count_right) = pq.pop().unwrap();
            pq.push(Self {
                children: vec![left, right],
                character: None,
            }, Reverse(count_left.0 + count_right.0));
        }
        pq.pop().unwrap().0
    }

    fn decrypt_char(&self, code: &[bool]) -> Option<char> {
        match code.split_first() {
            Some((first, rest)) => {
                if *first {
                    self.children.get(1).and_then(|child| child.decrypt_char(rest))
                } else {
                    self.children.get(0).and_then(|child| child.decrypt_char(rest))
                }
            }
            None => self.character,
        }
    }

    fn build_map(&self, current_path: Vec<bool>, map: &mut HashMap<char, Vec<bool>>) {
        match self.character {
            Some(c) => {
                map.insert(c, current_path);
            }
            None => {
                self.children[0].build_map({
                    let mut path = current_path.clone();
                    path.push(false);
                    path
                }, map);
                self.children[1].build_map({
                    let mut path = current_path.clone();
                    path.push(true);
                    path
                }, map);
            }
        }
    }

    pub fn encrypt(input: &str) -> HuffmanResult {
        let tree = Self::build_tree(input);
        let mut lookup = HashMap::new();
        tree.build_map(vec![], &mut lookup);

        let (final_indx, data) = input
            .chars()
            .flat_map(|c| lookup.get(&c).unwrap())
            .fold((0,Vec::new()), |(indx, mut acc), c|{
                if indx % 8 == 0 {
                    acc.push(if *c {1u8} else {0u8});
                } else if *c {
                    *acc.last_mut().unwrap() |= 1 << (indx % 8);
                }
                (indx + 1, acc)
            });

        HuffmanResult {
            tree,
            unused_bits: (8 - (final_indx + 1) % 8) as u8,
            data,
        }
    }
    
    pub fn decrypt(content: &HuffmanResult) -> String {
        let tree = &content.tree;
        let rest = &content.data;
        let unused = content.unused_bits;
        let mut result = Vec::new();
        let mut input = Vec::new();
        for i in 0..rest.len() * 8 - unused as usize {
            let indx = i / 8;
            let bit = (i % 8) as u8;
            input.push(rest[indx] & (1 << bit) != 0);
            if let Some(char) = tree.decrypt_char(&input) {
                result.push(char);
                input.clear();
            }
        }
        result.into_iter().collect()
    }  
}