use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LZ77 {
    data: Vec<u8>,
    look_ahead_buffer_size: u8,
    search_buffer_size: u8,
}

impl LZ77 {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
    
    pub fn deserialize(input: &[u8]) -> Self {
        bincode::deserialize(input).unwrap()
    }

    pub fn encode(input: &[u8]) -> LZ77 {
        let look_ahead_buffer_size = 255u8;
        let search_buffer_size = 255u8;


        let input_iter = &mut input.iter();
        
        let mut look_ahead_buffer: VecDeque<&u8> = input_iter.take(look_ahead_buffer_size as usize).collect::<VecDeque<&u8>>();
        let mut search_buffer: VecDeque<&u8> = VecDeque::new();
        
        let mut table: Vec<table_entry> = Vec::new();

        let mut best_offset = 0;
        let mut best_length = 0;
        while look_ahead_buffer.len() > 0 {
            best_offset = 0;
            best_length = 0;
            for i in 0..search_buffer.len() {
                for j in 0..look_ahead_buffer.len() {
                    if i + j >= search_buffer.len() {
                        break;
                    }
                    if search_buffer[i + j] != look_ahead_buffer[j] {
                        if j > best_length {
                            best_length = j;
                            best_offset = i;
                        }
                        break;   
                    }
                    if i + j == search_buffer.len() - 1 || j == look_ahead_buffer.len() - 1 {
                        if j + 1 > best_length {
                            best_length = j + 1;
                            best_offset = i;
                        }
                    }
                }
            }
            if best_length > 0 {
                for _ in 0..best_length {
                    if search_buffer.len() >= (search_buffer_size - 1) as usize {
                        search_buffer.pop_front();
                    }
                    if let Some(c) = look_ahead_buffer.pop_front() {
                        search_buffer.push_back(c);
                    }
                    if let Some(c) = input_iter.next() {
                        look_ahead_buffer.push_back(c);
                    }   
                }
            } 
            if let Some(&c) = look_ahead_buffer.front() {
                table.push(table_entry {
                    offset: best_offset as u8,
                    length: best_length as u8,
                    next_char: *c,
                });
            }
            if search_buffer.len() >= (search_buffer_size - 1) as usize {
                search_buffer.pop_front();
            }
            if let Some(c) = look_ahead_buffer.pop_front() {
                search_buffer.push_back(c);
            }
            if let Some(c) = input_iter.next() {
                look_ahead_buffer.push_back(c);
            }
        }

        table.push(table_entry {
            offset: best_offset as u8,
            length: best_length as u8,
            next_char: 0,
        });

        let data = table.iter().flat_map(|entry| {
            if entry.length == 0 && entry.offset == 0 {
                vec![0, entry.next_char]
            } else {
                vec![entry.length, entry.offset, entry.next_char]
            }
        }).collect::<Vec<u8>>();
        LZ77 {
            data,
            look_ahead_buffer_size,
            search_buffer_size,
        }
    }

    pub fn decode(&self) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut indx = 0;
        while indx < self.data.len() {
            if self.data[indx] == 0 {
                decompressed.push(table_entry {offset: 0, length: 0, next_char: self.data[indx + 1]});
                indx += 2;
            } else {
                decompressed.push(table_entry {offset: self.data[indx+1], length: self.data[indx], next_char: self.data[indx + 2]});
                indx += 3;
            }
        }

        let mut result = Vec::new();

        for entry in decompressed.iter() {
            if entry.length == 0 && entry.offset == 0 {
                result.push(entry.next_char);
            } else {
                let offset = entry.offset as usize;
                let length = entry.length as usize;
                let result_length_offset = (0).max(result.len() as i32 - self.search_buffer_size as i32 + 1) as usize;
                for i in 0..length {
                    let c = result[i + offset + result_length_offset];
                    result.push(c);
                }
                result.push(entry.next_char);
            }
        }
        result.pop();
        result
    }

}

#[derive(Debug)]
pub struct table_entry {
    pub offset: u8,
    pub length: u8,
    pub next_char: u8,
}