use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gold(u64);

impl Gold {
    pub fn new(value: u64) -> Gold {
        Gold(value)
    }
}

impl fmt::Display for Gold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} gold", self.0)
    }
}

impl Add for Gold {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Gold::new(self.0 + other.0)
    }
}

impl AddAssign for Gold {
    fn add_assign(&mut self, other: Self) {
        *self = Gold::new(self.0 + other.0);
    }
}

impl Sub for Gold {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Gold::new(self.0 - other.0)
    }
}

impl SubAssign for Gold {
    fn sub_assign(&mut self, other: Self) {
        *self = Gold::new(self.0 - other.0);
    }
}
