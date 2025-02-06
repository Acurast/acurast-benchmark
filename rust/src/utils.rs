use std::{
    ops::Add,
    slice,
    time::{Duration, Instant},
};

macro_rules! vec_with_len {
    ($n: expr) => {{
        let mut vec = Vec::with_capacity($n);
        unsafe { vec.set_len($n) };

        vec
    }};
}

pub(crate) use vec_with_len;

pub(crate) const KB: usize = 1024;
pub(crate) const MB: usize = KB * KB;
pub(crate) const GB: usize = KB * MB;

pub(crate) fn slice_from_ptr_mut<'a, T>(ptr: *mut T, from: usize, until: usize) -> &'a mut [T] {
    unsafe { slice::from_raw_parts_mut(ptr.add(from), until - from) }
}

pub(crate) fn is_pow(n: usize, pow: usize) -> bool {
    let mut n = n as f64;
    let pow = pow as f64;

    while n > 1f64 {
        n /= pow;
        if n.fract() != 0.0 {
            return false;
        }
    }

    true
}

pub(crate) fn closest_pow(n: usize, pow: usize) -> usize {
    if n == 0 {
        return 0;
    }

    let mut upper = 1;
    while upper < n {
        upper *= pow;
    }

    let lower = upper / pow;

    if n - lower < upper - n {
        lower
    } else {
        upper
    }
}

pub(crate) trait GetValue {
    type Value;

    fn value(&self) -> &Self::Value;
}

impl<T> GetValue for Result<T, T> {
    type Value = T;

    fn value(&self) -> &Self::Value {
        match self {
            Ok(v) => v,
            Err(v) => v,
        }
    }
}

pub(crate) trait AddValue {
    type Value;

    fn add(self, value: Self::Value) -> Self;
}

impl<T> AddValue for Result<T, T>
where
    T: Add<Output = T>,
{
    type Value = T;

    fn add(self, value: Self::Value) -> Self {
        match self {
            Ok(v) => Ok(v + value),
            Err(v) => Err(v + value),
        }
    }
}

pub(crate) trait Avg {
    type T;

    fn avg(&self) -> Self::T;
}

impl Avg for Vec<Duration> {
    type T = Duration;

    fn avg(&self) -> Self::T {
        if self.len() == 0 {
            return Duration::ZERO;
        }

        self.iter().sum::<Duration>() / (self.len() as u32)
    }
}

pub(crate) struct Timeout {
    pub(crate) start: Instant,
    pub(crate) duration: Duration,
}

pub(crate) trait Expirable {
    fn reached(&self) -> bool;

    fn reached_with_err<T>(&self, err: T) -> Result<(), T> {
        if self.reached() {
            return Err(err);
        }

        Ok(())
    }
}

impl Timeout {
    pub(crate) fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }
}

impl Expirable for Timeout {
    fn reached(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}

impl Expirable for Option<&Timeout> {
    fn reached(&self) -> bool {
        match self {
            Some(timeout) => timeout.reached(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_from_ptr_mut() {
        let mut data = Vec::from_iter(0..32u8);
        let slice = data.as_mut_slice();

        assert_eq!(
            &[0, 1, 2, 3, 4, 5],
            slice_from_ptr_mut(slice.as_mut_ptr(), 0, 6)
        );
        assert_eq!(
            &[14, 15, 16],
            slice_from_ptr_mut(slice.as_mut_ptr(), 14, 17)
        );
        assert_eq!(&[30, 31], slice_from_ptr_mut(slice.as_mut_ptr(), 30, 32));
    }

    #[test]
    fn test_is_pow() {
        assert_eq!(true, is_pow(1, 2));
        assert_eq!(true, is_pow(2, 2));
        assert_eq!(true, is_pow(64, 2));
        assert_eq!(true, is_pow(256, 2));

        assert_eq!(false, is_pow(3, 2));
        assert_eq!(false, is_pow(10, 2));
        assert_eq!(false, is_pow(100, 2));
    }

    #[test]
    fn test_closest_pow() {
        assert_eq!(0, closest_pow(0, 2));
        assert_eq!(1, closest_pow(1, 2));
        assert_eq!(8, closest_pow(6, 2));
        assert_eq!(32, closest_pow(47, 2));
    }
}
