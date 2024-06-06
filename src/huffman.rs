use std::{cmp::Reverse, collections::HashMap};
use priority_queue::PriorityQueue;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Huffman {
    children: Vec<Huffman>,
    character: Option<char>,
}

impl Huffman {
    fn serialize(&self) -> String {
        match self.character {
            Some(c) => c.to_string(),
            None => format!("警{}{}", self.children[0].serialize(), self.children[1].serialize()),
        }
    }

    fn deserialize(input: String) -> (Huffman, String) {
        let (node, s) = Huffman::deserialize_rec(input.chars().collect::<Vec<char>>());
        (node, s.iter().collect::<String>())
    }

    fn deserialize_rec(input: Vec<char>) -> (Huffman, Vec<char>) {
        if input[0] == '警' {
            let (left, rest) = Huffman::deserialize_rec(input[1..].to_vec());
            let (right, rest) = Huffman::deserialize_rec(rest);
            (Huffman {
                children: vec![left, right],
                character: None,
            }, rest)
        } else {
            (Huffman {
                children: vec![],
                character: Some(input[0]),
            }, input[1..].to_vec())
        }
        
    }

    fn build_tree(input: &str) -> Huffman {
        let chars_counts = input.chars()
            .fold(HashMap::new(), |mut map, c|{
                *map.entry(c).or_insert(0) += 1;
                map
            })
            .into_iter()
            .collect::<Vec<(char, i32)>>();

        let mut pq: PriorityQueue<Huffman, _, _> = PriorityQueue::new();
        pq.extend(chars_counts.into_iter().map(|(c, count)| (Huffman {
            children: vec![],
            character: Some(c),
        }, Reverse(count))));

        while pq.len() > 1 {
            let (left, count_left) = pq.pop().unwrap();
            let (right, count_right) = pq.pop().unwrap();
            pq.push(Huffman {
                children: vec![left, right],
                character: None,
            }, Reverse(count_left.0 + count_right.0));
        }
        pq.pop().unwrap().0
    }

    fn encrypt_char(&self, code: &[bool]) -> Option<char> {
        match code.split_first() {
            Some((first, rest)) => {
                if *first {
                    self.children.get(1).and_then(|child| child.encrypt_char(rest))
                } else {
                    self.children.get(0).and_then(|child| child.encrypt_char(rest))
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

    pub fn encrypt(input: &str) -> String {
        let tree = Huffman::build_tree(input);

        let mut lookup = HashMap::new();
        tree.build_map(vec![], &mut lookup);

        let serialized_tree = tree.serialize();
        let serialized_data = input.chars().flat_map(|c| lookup.get(&c).unwrap())
            .collect::<Vec<_>>()
            .chunks(8)
            .map(|chunk| {
                let mut byte:u8 = 0;
                for (i, bit) in chunk.iter().enumerate() {
                    if **bit {
                        byte |= 1 << (7 - i);
                    }
                }
                byte as char
            })
            .collect::<String>();

        format!("{}{}", serialized_tree, serialized_data)
    }

    pub fn decrypt(input: String) -> String {
        let (tree, rest) = Huffman::deserialize(input);
        let input = rest.chars().flat_map(|c| {
            (0..8).rev().map(move |i| c as u8 & (1 << i as u8) != 0)
        }).collect::<Vec<bool>>();
        let mut start = 0;
        let mut result = Vec::new();
        for i in 0..input.len()  {
            if let Some(char) = tree.encrypt_char(&input[start..=i]) {
                result.push(char);
                start = i + 1;
            }
        }
        result.into_iter().collect()
    }   

}
