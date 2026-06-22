#[derive(Clone)]
pub struct BitField {
    bits: Vec<u64>,
}

impl BitField {
    pub fn new(num_bits: usize) -> Self {
        let num_u64s = (num_bits + 63) / 64;
        Self {
            bits: vec![0u64; num_u64s],
        }
    }

    pub fn from_vec(data: Vec<u8>) -> Self {
        let bits: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                let mut arr = [0u8; 8];
                arr[..chunk.len()].copy_from_slice(chunk);
                u64::from_be_bytes(arr)
            })
            .collect();

        Self { bits }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bits
            .iter()
            .flat_map(|&chunk| chunk.to_be_bytes())
            .collect()
    }

    pub fn get_bit(&self, index: usize) -> u64 {
        let chunk_idx = index / 64;
        let bit_idx = index % 64;
        
        if chunk_idx >= self.bits.len() {
            return 0;
        }
        
        (self.bits[chunk_idx] >> bit_idx) & 1
    }

    pub fn set_bit(&mut self, index: usize, value: u64) {
        let chunk_idx = index / 64;
        let bit_idx = index % 64;
        
        if chunk_idx >= self.bits.len() {
            return;
        }
        
        if value == 1 {
            self.bits[chunk_idx] |= 1 << bit_idx;
        } else {
            self.bits[chunk_idx] &= !(1 << bit_idx);
        }
    }

    pub fn len(&self) -> usize {
        self.bits.len() * 64
    }
}