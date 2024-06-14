use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use crate::bitbuffer::{self, BitBuffer};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LZ77 {
    pub bitbuffers: Vec<bitbuffer::BitBuffer>,
}

impl LZ77 {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
    
    pub fn deserialize(input: &[u8]) -> Self {
        bincode::deserialize(input).unwrap()
    }

    fn lpc(input: &[u8], i: usize, j: usize) -> usize {
        if i == 0 || j == 0 {
            return 0;
        }
        let mut k = 0;
        while j + k < input.len() && i + k < input.len() && input[i + k] == input[j + k] {
            k += 1;
        }
        k
    }

    pub fn fast_encode(input: &[u8], bits: u8) -> BitBuffer {
        let n = input.len();

        let mut suffix_array = (0..=n).collect::<Vec<usize>>();
        suffix_array.sort_unstable_by_key(|&i| &input[i..]);

        let mut inverse_suffix_array = vec![0; n+1];
        for (i, suffix_indx) in suffix_array.iter().enumerate() {
            inverse_suffix_array[*suffix_indx] = i;
        }

        let mut nsv = vec![0; n+1];
        let mut psv = vec![usize::MAX; n+1];
        for i in 1..n {
            let mut j = i - 1;
            while psv[j] != usize::MAX && suffix_array[i] < suffix_array[j] {
                nsv[j] = i;
                j = psv[j];
            }
            psv[i] = j;
        }
        psv = psv.into_iter().map(|i| if i == usize::MAX {0} else {i}).collect::<Vec<_>>();
        nsv = nsv.into_iter().map(|i| i).collect::<Vec<_>>();

        let mut factors = Vec::new();
        let mut k = 0;
        while k < n {
            let psv = suffix_array[psv[inverse_suffix_array[k]]];
            let nsv = suffix_array[nsv[inverse_suffix_array[k]]];
            let (p,l,c,indx) = LZ77::lz_factor(k, psv, nsv, input);
            k = indx;
            factors.push((p,l,c));
        }
        
        factors.into_iter().fold(BitBuffer::new(), | mut acc ,(mut p,mut l,c)| {
            if l == 0 {
                acc.write_byte(0);
                acc.write_byte(c);
            } else if l < u8::MAX as usize {
                acc.write_byte(l as u8);
                acc.write_bits(p as u32, bits);
            } else {
                while l >= u8::MAX as usize {
                    acc.write_byte(u8::MAX);
                    acc.write_bits(p as u32, bits);
                    p += u8::MAX as usize;
                    l -= u8::MAX as usize;
                }
                if l != 0 {
                    acc.write_byte(l as u8);
                    acc.write_bits(p as u32, bits);
                }
            }
            acc
        })
    }

    fn decode_chunk(factors: &Vec<(usize, usize, u8)>) -> Vec<u8> {
        let mut decoded = Vec::new();
        for (p,l,c) in factors {
            if *l == 0 {
                decoded.push(*c);
            } else {
                for i in 0..*l {
                    let c = decoded[p + i];
                    decoded.push(c);
                }
            }
        }
        decoded
    }

    fn lz_factor(i:usize, psv: usize, nsv: usize, x: &[u8]) -> (usize, usize, u8, usize) {
        let v1 = LZ77::lpc(x, i, psv);
        let v2 = LZ77::lpc(x, i, nsv);
        let (mut p,l) = if v1 > v2 {
            (psv, v1)
        } else {
            (nsv, v2)
        };
        if l == 0 {
            p = i;
        }
        let e =  x.get(i + l).unwrap_or(&0);
        (p, l, *e, i + l.max(1))
    }

    pub fn encode(input: &[u8], bits: u8) -> LZ77 {
        let n = input.len();
        let chunk_size = 2usize.pow(bits as u32) - 1;
        let num_chunks = n / chunk_size + if n % chunk_size == 0 {0} else {1};

        let data = (0..num_chunks).into_par_iter()
            .map(|i| {
                let start = i * chunk_size;
                let end = usize::min((i + 1) * chunk_size, n);
                let chunk = &input[start..end];
                let factors = LZ77::fast_encode(chunk, bits);
                factors
            })
            .collect::<Vec<_>>();

        LZ77 {
            bitbuffers: data,
        }
    }

    pub fn decode(&mut self, bits: u8) -> Vec<u8> {
        self.bitbuffers.par_iter_mut().flat_map(|chunk| {
            let mut factors = Vec::new();
            while let Some(l) = chunk.read_byte() {
                if l == 0 {
                    factors.push((0, 0, chunk.read_byte().unwrap()));
                } else {
                    factors.push((chunk.read_bits(bits).unwrap() as usize, l as usize, 0));              
                }
            }
            LZ77::decode_chunk(&factors)
        }).collect::<Vec<_>>()
    }

}
