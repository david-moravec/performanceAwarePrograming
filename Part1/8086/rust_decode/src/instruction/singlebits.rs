// Encodes S, W, D, V, Z bits
pub struct SingleBits(u8);

impl SingleBits {
    const S_MASK: u8 = 0b10000;
    const W_MASK: u8 = 0b01000;
    const D_MASK: u8 = 0b00100;
    const V_MASK: u8 = 0b00010;
    const Z_MASK: u8 = 0b00001;

    pub fn new() -> Self {
        SingleBits(0)
    }

    pub fn s(&self) -> u8 {
        self.0 & Self::S_MASK
    }

    pub fn w(&self) -> u8 {
        self.0 & Self::W_MASK
    }

    pub fn d(&self) -> u8 {
        self.0 & Self::D_MASK
    }

    pub fn v(&self) -> u8 {
        self.0 & Self::V_MASK
    }

    pub fn z(&self) -> u8 {
        self.0 & Self::Z_MASK
    }

    pub fn set_s(&mut self) {
        self.0 |= Self::S_MASK
    }

    pub fn set_w(&mut self) {
        self.0 |= Self::W_MASK
    }

    pub fn set_d(&mut self) {
        self.0 |= Self::D_MASK
    }

    pub fn set_v(&mut self) {
        self.0 |= Self::V_MASK
    }

    pub fn set_z(&mut self) {
        self.0 |= Self::Z_MASK
    }
}
