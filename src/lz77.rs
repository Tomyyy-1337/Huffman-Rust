use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct LZ77 {
    pub data: Vec<u8>,
    pub chunk_sizes: Vec<u32>,
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

    pub fn fast_encode(input: &[u8], pos_bytes: u32) -> Vec<u8>{
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
        
        factors.into_iter().flat_map(|(mut p,mut l,c)| {
            if l == 0 {
                vec![0, c]
            } else if l < u8::MAX as usize {
                let p_bytes = p.to_le_bytes();
                if pos_bytes == 3 {
                    vec![l as u8,p_bytes[0], p_bytes[1], p_bytes[2]]
                } else {
                    vec![l as u8,p_bytes[0], p_bytes[1]]
                }
            } else {
                let mut result = Vec::new();
                while l >= u8::MAX as usize {
                    let p_bytes = p.to_le_bytes();
                    if pos_bytes == 3 {
                        result.extend([u8::MAX, p_bytes[0], p_bytes[1], p_bytes[2]]);
                    } else {
                        result.extend([u8::MAX, p_bytes[0], p_bytes[1]]);
                    }
                    p += u8::MAX as usize;
                    l -= u8::MAX as usize;
                }
                if l != 0 {
                    let p_bytes = p.to_le_bytes();
                    if pos_bytes == 3 {
                        result.extend([l as u8, p_bytes[0], p_bytes[1], p_bytes[2]]);
                    } else {
                        result.extend([l as u8, p_bytes[0], p_bytes[1]]);
                    }
                }
                result
            }
        }).collect::<Vec<_>>()
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

    pub fn encode(input: &[u8], pos_bytes: u32) -> LZ77 {
        let n = input.len();
        let chunk_size = 2usize.pow(pos_bytes * 8);

        let num_chunks = (n + chunk_size - 1) / chunk_size;

        let data = (0..num_chunks).into_par_iter()
            .map(|i| {
                let start = i * chunk_size;
                let end = usize::min((i + 1) * chunk_size, n);
                let chunk = &input[start..end];
                let factors = LZ77::fast_encode(chunk, pos_bytes);
                factors
            })
            .collect::<Vec<Vec<u8>>>();

        LZ77 {
            chunk_sizes: data.iter().map(|x| x.len() as u32).collect(),
            data: data.into_iter().flatten().collect(),
        }
    }

    pub fn decode(&self, pos_bytes: u32) -> Vec<u8> {
        let chunks = self.chunk_sizes.iter().scan(0, |state, &x| {
            let start = *state;
            *state += x as usize;
            Some(&self.data[start..*state])
        }).collect::<Vec<_>>();
        chunks.par_iter().flat_map(|chunk| {
            let mut indx = 0;
            let mut factors = Vec::new();
            while indx < chunk.len() {
                if chunk[indx] == 0 {
                    factors.push((0, 0, chunk[indx + 1]));
                    indx += 2;
                } else {
                    if pos_bytes == 3 {
                        factors.push((u32::from_le_bytes([chunk[indx + 1], chunk[indx + 2], chunk[indx + 3], 0]) as usize, chunk[indx] as usize, 0));           
                    } else { // = 2
                        factors.push((u32::from_le_bytes([chunk[indx + 1], chunk[indx + 2], 0, 0]) as usize, chunk[indx] as usize, 0));           
                    }
                    indx += pos_bytes as usize + 1;
                }
            }
            LZ77::decode_chunk(&factors)
        }).collect::<Vec<_>>()
    }

}
