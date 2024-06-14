use std::{cmp::Reverse, collections::HashMap};

use priority_queue::PriorityQueue;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Huffman {
    tree: HuffmanTree,
    unused_bits: u8,
    pub data: Vec<u8>,
}

impl Huffman {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn encrypt(input: &Vec<u8>) -> Huffman {
        let tree = HuffmanTree::build_tree(&input);
        let mut lookup = HashMap::new();
        tree.build_map(vec![], &mut lookup);

        let (count, data) = input
            .into_iter()
            .flat_map(|c| lookup.get(&c).unwrap())
            .fold((0usize,Vec::new()), |(indx, mut acc), c|{
                if indx % 8 == 0 {
                    acc.push(if *c {1u8} else {0u8});
                } else if *c {
                    *acc.last_mut().unwrap() |= 1 << (indx % 8);
                }
                (indx + 1, acc)
            });

        Huffman {
            tree,
            unused_bits: match count % 8 {
                0 => 0,
                n => 8 - n as u8,
            },
            data,
        }
    }

    pub fn decrypt(&self) -> Vec<u8> {
        let tree = &self.tree;
        let data = &self.data;
        let unused = self.unused_bits;
        let mut result = Vec::new();
        let mut input = Vec::new();
        for i in 0..data.len() * 8 - unused as usize {
            let indx = i / 8;
            let bit = (i % 8) as u8;
            input.push(data[indx] & (1 << bit) != 0);
            if let Some(char) = tree.decrypt_char(&input) {
                result.push(char);
                input.clear();
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct HuffmanTree {
    children: Vec<HuffmanTree>,
    character: Option<u8>,
}

impl HuffmanTree {
    fn build_tree(input: &Vec<u8>) -> HuffmanTree {
        let mut counts = [0u64;256];

        for &e in input {
            counts[e as usize] += 1;
        }

        let mut pq: PriorityQueue<Self, _, _> = PriorityQueue::new();
        pq.extend(counts.into_iter().enumerate().map(|(c, count)| (Self {
            children: vec![],
            character: Some(c as u8),
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

    fn decrypt_char(&self, code: &[bool]) -> Option<u8> {
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

    fn build_map(&self, current_path: Vec<bool>, map: &mut HashMap<u8, Vec<bool>>) {
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
}