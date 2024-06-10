use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LZ77 {
    pub data: Vec<u8>,
}

impl LZ77 {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
    
    pub fn deserialize(input: &[u8]) -> Self {
        bincode::deserialize(input).unwrap()
    }

    pub fn from_data(data: Vec<u8>) -> Self {
        LZ77 {
            data,
        }
    }

    pub fn encode(input: &[u8]) -> LZ77 {
        let look_ahead_buffer_size = 255u8;
        let search_buffer_size = u16::MAX;

        let input_iter = &mut input.iter();
        
        let mut look_ahead_buffer: VecDeque<&u8> = input_iter.take(look_ahead_buffer_size as usize).collect::<VecDeque<&u8>>();
        let mut search_buffer: VecDeque<&u8> = VecDeque::new();
        
        let mut table: Vec<table_entry> = Vec::new();

        let mut best_offset = 0;
        let mut best_length = 0;

        while look_ahead_buffer.len() > 0 {
            best_offset = 0;
            best_length = 0;

            let first = look_ahead_buffer[0];
            let mut possible_offsets = search_buffer.iter().enumerate()
                .filter(|&(_,&elem)| elem == first)
                .map(|(i,_)| i as u16)
                .collect::<Vec<_>>();
            while possible_offsets.len() > 0 {
                best_offset = possible_offsets[0];
                best_length += 1;
                possible_offsets.retain(|&offset| 
                    offset + best_length < search_buffer.len() as u16 
                    && best_length < look_ahead_buffer.len() as u16 
                    && search_buffer[(offset + best_length) as usize] == look_ahead_buffer[best_length as usize] 
                );
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
                    offset: best_offset as u16,
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
            offset: best_offset as u16,
            length: best_length as u8,
            next_char: 0,
        });

        let data = table.iter().flat_map(|entry| {
            if entry.length == 0 && entry.offset == 0 {
                vec![0, entry.next_char]
            } else {
                let [byte1, byte2] = entry.offset.to_ne_bytes();
                vec![entry.length, byte1, byte2, entry.next_char]
            }
        }).collect::<Vec<u8>>();
        LZ77 {
            data,
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
                decompressed.push(table_entry {offset: u16::from_ne_bytes([self.data[indx+1], self.data[indx+2]]), length: self.data[indx], next_char: self.data[indx + 3]});
                indx += 4;
            }
        }

        let mut result = Vec::new();

        for entry in decompressed.iter() {
            if entry.length == 0 && entry.offset == 0 {
                result.push(entry.next_char);
            } else {
                let offset = entry.offset as usize;
                let length = entry.length as usize;
                let result_length_offset = (0).max(result.len() as i32 - u16::MAX as i32 + 1) as usize;
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
    pub offset: u16,
    pub length: u8,
    pub next_char: u8,
}