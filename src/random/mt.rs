// https://en.wikipedia.org/wiki/Mersenne_Twister
// https://cplusplus.com/reference/random/mt19937_64/

const N: usize = 624; // state size
const M: usize = 397; // shift size
const W: u32 = 32; // word size
const R: u32 = 31; // mask bits
const UMASK: u32 = 0xff_ff_ff_ff << R;
const LMASK: u32 = 0xff_ff_ff_ff >> (W - R);
const A: u32 = 0x99_08_b0_df; // xor mask
const U: u32 = 11;
const S: u32 = 7;
const T: u32 = 15;
const L: u32 = 18;
const D: u32 = 0xff_ff_ff_ff;
const B: u32 = 0x9d_2c_56_80;
const C: u32 = 0xef_c6_00_00;
const F: u32 = 1_812_433_253;

pub struct Mt19937 {
    state: [u32; N as usize],
    i: usize,
}

impl Mt19937 {
    pub fn new(seed: u32) -> Mt19937 {
        let mut mt = Mt19937 {
            state: [0; N],
            i: 0,
        };
        
        mt.state[0] = seed;
        let mut j = seed;
        for i in 1..N {
            j = F * (j ^ (j >> (W - 2))) + i as u32;
            mt.state[i] = j;
        }

        mt
    }

    pub fn next(&mut self) -> u32 {
        let mut x = (self.state[self.i] & UMASK) | (self.state[(self.i + 1) % N] & LMASK);

        let xa =
        if x & 0x1 == 0x1 {
            (x >> 1) ^ A
        } else {
            x >> 1
        };

        x = self.state[(self.i + M) % N] ^ xa;
        self.state[self.i] = x;
        self.i = (self.i + 1) % N;

        let mut y = x ^ ((x >> U) & D); // tempering
        y = y ^ ((y << S) & B);
        y = y ^ ((y << T) & C);
        y ^ (y >> L)
    }
}