// https://en.wikipedia.org/wiki/Mersenne_Twister
// https://cplusplus.com/reference/random/mt19937/

pub struct Mt19937 {
    state: [u32; Self::N as usize],
    i: usize,
}

impl Mt19937 {
    const N: usize = 624; // state size
    const M: usize = 397; // shift size
    const W: u32 = 32; // word size
    const R: u32 = 31; // mask bits
    const UMASK: u32 = 0xff_ff_ff_ff << Self::R;
    const LMASK: u32 = 0xff_ff_ff_ff >> (Self::W - Self::R);
    const A: u32 = 0x99_08_b0_df; // xor mask
    const U: u32 = 11;
    const S: u32 = 7;
    const T: u32 = 15;
    const L: u32 = 18;
    const D: u32 = 0xff_ff_ff_ff;
    const B: u32 = 0x9d_2c_56_80;
    const C: u32 = 0xef_c6_00_00;
    const F: u32 = 1_812_433_253;

    pub fn new(seed: u32) -> Mt19937 {
        let mut mt = Mt19937 {
            state: [0; Self::N],
            i: 0,
        };
        
        mt.state[0] = seed;
        let mut j = seed;
        for i in 1..Self::N {
            j = Self::F.wrapping_mul(j ^ (j >> (Self::W - 2))).wrapping_add(i as u32);
            mt.state[i] = j;
        }

        mt
    }

    pub fn next(&mut self) -> u32 {
        let mut x = (self.state[self.i] & Self::UMASK) | (self.state[(self.i + 1) % Self::N] & Self::LMASK);

        let xa =
        if x & 0x1 == 0x1 {
            (x >> 1) ^ Self::A
        } else {
            x >> 1
        };

        x = self.state[(self.i + Self::M) % Self::N] ^ xa;
        self.state[self.i] = x;
        self.i = (self.i + 1) % Self::N;

        let mut y = x ^ ((x >> Self::U) & Self::D); // tempering
        y = y ^ ((y << Self::S) & Self::B);
        y = y ^ ((y << Self::T) & Self::C);
        y ^ (y >> Self::L)
    }

    // single precision float:
    // 1 sign, 8 exponent, 23 fraction
    // v = (-1)^sign * 2^(e-127) * 1.fffffff..
    pub fn next_01(&mut self) -> f32 {
        let rand_int = self.next();
        
        // look at the first 9 bits of `rand_int` to determine the exponent
        let mut expon: u32 = 126;
        for i in 0..9 {
            if (rand_int >> (31 - i)) & 0x1 == 0x1 {
                expon -= 1;
            } else {
                break;
            }
        }

        // exponent should be in range [126 - 9, 126]
        assert!(126 - 9 <= expon && expon <= 126);

        // assemble the f32
        let mask = 0xff_ff_ff_ff >> 9;
        f32::from_bits((expon << 23) | (rand_int & mask))
    }
}