#[allow(dead_code)]

pub const SIZE: usize = 32;

pub trait VScaleTestData {
    fn name() -> &'static str;
    fn zero() -> Self;
    fn scale() -> Self;
    fn input() -> Self;
    fn output() -> Self;
}

impl VScaleTestData for u32 {
    fn name() -> &'static str {
        "u32"
    }
    fn zero() -> u32 {
        0
    }
    fn scale() -> u32 {
        6
    }
    fn input() -> u32 {
        7
    }
    fn output() -> u32 {
        42
    }
}

impl VScaleTestData for i32 {
    fn name() -> &'static str {
        "i32"
    }
    fn zero() -> i32 {
        0
    }
    fn scale() -> i32 {
        6
    }
    fn input() -> i32 {
        7
    }
    fn output() -> i32 {
        42
    }
}

impl VScaleTestData for i64 {
    fn name() -> &'static str {
        "i64"
    }
    fn zero() -> i64 {
        0
    }
    fn scale() -> i64 {
        6
    }
    fn input() -> i64 {
        7
    }
    fn output() -> i64 {
        42
    }
}

impl VScaleTestData for u64 {
    fn name() -> &'static str {
        "u64"
    }
    fn zero() -> u64 {
        0
    }
    fn scale() -> u64 {
        6
    }
    fn input() -> u64 {
        7
    }
    fn output() -> u64 {
        42
    }
}

impl VScaleTestData for f32 {
    fn name() -> &'static str {
        "f32"
    }
    fn zero() -> f32 {
        0.0
    }
    fn scale() -> f32 {
        6.0
    }
    fn input() -> f32 {
        7.0
    }
    fn output() -> f32 {
        42.0
    }
}

impl VScaleTestData for f64 {
    fn name() -> &'static str {
        "f64"
    }
    fn zero() -> f64 {
        0.0
    }
    fn scale() -> f64 {
        6.0
    }
    fn input() -> f64 {
        7.0
    }
    fn output() -> f64 {
        42.0
    }
}
