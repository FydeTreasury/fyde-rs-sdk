use ethers::prelude::U256;

pub trait ToF32 {
    fn to_f32(self, divisor: f32) -> f32;
}

impl ToF32 for U256 {
    fn to_f32(self, decimals: f32) -> f32 {
        self.as_u128() as f32 / 10f32.powf(decimals)
    }
}
