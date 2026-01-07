use std::slice::from_raw_parts_mut;

#[repr(C)]
pub struct Philox64x2 {
    c: [u64; 2],
    k: [u64; 2],
}

impl Philox64x2 {
    const fn chunk_size() -> usize {
        2
    }

    const fn m0() -> u64 {
        0xD2B74407B1CE6E93
    }

    fn new(seed: [u64; 2]) -> Self {
        Self { c: [1, 0], k: seed }
    }

    fn next(&mut self) -> [u64; 2] {
        let mut out = [0u64; 2];
        let prod = (self.c[0] as u128).wrapping_mul(Self::m0() as u128);

        out[0] = (prod as u64) ^ self.c[1] ^ self.k[0];
        out[1] = (prod >> 64) as u64;

        self.c[0] = self.c[0].wrapping_add(1);
        self.c[1] = self.c[1].wrapping_add(1);

        out
    }
}

#[unsafe(no_mangle)]
pub fn philox64x2_new(seed1: u64, seed2: u64) -> *mut Philox64x2 {
    Box::into_raw(Box::new(Philox64x2::new([seed1, seed2])))
}

#[unsafe(no_mangle)]
pub fn philox64x2_free(ptr: *mut Philox64x2) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) }
    }
}

#[unsafe(no_mangle)]
pub fn next_u64s(ptr: *mut Philox64x2, out: *mut u64, count: usize) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        for v in buffer.chunks_mut(Philox64x2::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = next[i];
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn next_f64s(ptr: *mut Philox64x2, out: *mut f64, count: usize) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        for v in buffer.chunks_mut(Philox64x2::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = next[i] as f64 / (u64::MAX as f64 + 1.);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn rand_i64s(ptr: *mut Philox64x2, out: *mut i64, count: usize, min: i64, max: i64) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        let range = (max as i128 - min as i128 + 1) as u128;
        for v in buffer.chunks_mut(Philox64x2::chunk_size()) {
            let next = rng.next();
            for (i, k) in v.iter_mut().enumerate() {
                *k = (next[i] as u128 % range) as i64 + min;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn rand_u64s(ptr: *mut Philox64x2, out: *mut u64, count: usize, min: f64, max: f64) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        let range = max - min;
        for v in buffer.chunks_mut(Philox64x2::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = ((next[i] as f64 / u64::MAX as f64) * range + min) as u64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Philox64x2;

    #[test]
    fn test_philox64x2() {
        let p64 = &mut Philox64x2::new([1, 2]);
        assert_eq!(p64.next(), [15183679468541472402, 0]);
    }
}
