use std::mem::MaybeUninit;

pub const SIGMA: usize = 256;

pub(crate) fn initialize<T>(f: impl Fn() -> T) -> [T; SIGMA] {
    unsafe {
        let mut t: [MaybeUninit<T>; SIGMA] =
            MaybeUninit::uninit().assume_init();
        for x in &mut t {
            *x = MaybeUninit::new(f());
        }
        std::mem::transmute_copy(&t)
    }
}
