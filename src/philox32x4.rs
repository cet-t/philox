use std::slice::from_raw_parts_mut;

#[repr(C)]
pub struct Philox32x4 {
    c: [u32; 4],
    k: [u32; 2],
}

impl Philox32x4 {
    const fn chunk_size() -> usize {
        4
    }

    const fn m0() -> u32 {
        0xD2511F53
    }

    const fn m1() -> u32 {
        0xCD9E8D57
    }

    pub(crate) fn new(seed: [u32; 2]) -> Self {
        Self {
            c: [1, 0, 0, 0],
            k: seed,
        }
    }

    pub(crate) fn next(&mut self) -> [u32; 4] {
        let mut out = [0u32; 4];

        out[0] = self.c[0].wrapping_mul(Self::m0()) ^ self.c[1] ^ self.k[0];
        out[1] = self.c[0].wrapping_mul(Self::m0());
        out[2] = self.c[2].wrapping_mul(Self::m1()) ^ self.c[3] ^ self.k[1];
        out[3] = self.c[2].wrapping_mul(Self::m1());

        self.c[0] = self.c[0].wrapping_add(1);
        self.c[1] = self.c[1].wrapping_add(1);

        out
    }
}

#[unsafe(no_mangle)]
pub fn philox32x4_new(seed1: u32, seed2: u32) -> *mut Philox32x4 {
    Box::into_raw(Box::new(Philox32x4::new([seed1, seed2])))
}

#[unsafe(no_mangle)]
pub fn philox32x4_free(ptr: *mut Philox32x4) {
    if !ptr.is_null() {
        unsafe { drop(Box::from_raw(ptr)) }
    }
}

#[unsafe(no_mangle)]
pub fn next_u32s(ptr: *mut Philox32x4, out: *mut u32, count: usize) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        for v in buffer.chunks_mut(Philox32x4::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = next[i];
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn next_f32s(ptr: *mut Philox32x4, out: *mut f32, count: usize) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        for v in buffer.chunks_mut(Philox32x4::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = next[i] as f32 / (u32::MAX as f32 + 1.);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn rand_i32s(ptr: *mut Philox32x4, out: *mut i32, count: usize, min: i32, max: i32) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        let range = (max - min + 1) as u64;
        for v in buffer.chunks_mut(Philox32x4::chunk_size()) {
            let next = rng.next();
            for (i, k) in v.iter_mut().enumerate() {
                *k = (next[i] as u64 % range) as i32 + min;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub fn rand_u32s(ptr: *mut Philox32x4, out: *mut u32, count: usize, min: f32, max: f32) {
    unsafe {
        let rng = &mut *ptr;
        let buffer = from_raw_parts_mut(out, count);
        let range = max - min;
        for v in buffer.chunks_mut(Philox32x4::chunk_size()) {
            let next = rng.next();
            for i in 0..v.len() {
                v[i] = ((next[i] as f32 / u32::MAX as f32) * range + min) as u32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Philox32x4;

    #[test]
    fn test_philox32x4() {
        let p32 = &mut Philox32x4::new([1, 2]);
        assert_eq!(p32.next(), [3528531794, 3528531795, 2, 0]);
    }
}
