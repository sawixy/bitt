pub struct BitField {
    bits: Vec<u64>,
}

impl BitField {
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
        if index/64 >= self.bits.len() { return 0; }
        self.bits[index/64] & (1 << index%64)
    }

    pub fn set_bit(&mut self, index: usize) {
        self.bits[index/64] |= 1 << index%64;
    }

    pub fn len(&self) -> usize {
        self.bits.len() * 64
    }
}