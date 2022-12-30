use derive_more::{Add, AddAssign, Neg, Sub, SubAssign};
use plegine::json::{self, FromValue};
use std::ops::Mul;

#[repr(transparent)]
#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd, Eq, Ord, Neg)]
pub struct Mass(pub i128);

#[repr(transparent)]
#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd, Eq, Ord, Neg)]
pub struct Volume(pub i128);

#[repr(transparent)]
#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign, PartialEq, PartialOrd, Eq, Ord)]
pub struct Density(pub i128);

impl Mul<i128> for Mass {
    type Output = Mass;

    fn mul(self, rhs: i128) -> Self::Output {
        Mass(self.0 * rhs)
    }
}

impl Mul<i128> for Volume {
    type Output = Volume;

    fn mul(self, rhs: i128) -> Self::Output {
        Volume(self.0 * rhs)
    }
}

impl Mul<i128> for Density {
    type Output = Density;

    fn mul(self, rhs: i128) -> Self::Output {
        Density(self.0 * rhs)
    }
}

impl Mul<Volume> for Density {
    type Output = Mass;

    fn mul(self, rhs: Volume) -> Self::Output {
        Mass(self.0 * rhs.0)
    }
}

impl Mul<Density> for Volume {
    type Output = Mass;

    fn mul(self, rhs: Density) -> Self::Output {
        Mass(self.0 * rhs.0)
    }
}

impl FromValue for Mass {
    fn from_value(value: json::Value) -> plegine::json::ParseResult<Self> {
        i128::from_value(value).map(Mass)
    }
}

impl FromValue for Volume {
    fn from_value(value: json::Value) -> plegine::json::ParseResult<Self> {
        i128::from_value(value).map(Volume)
    }
}

impl FromValue for Density {
    fn from_value(value: json::Value) -> plegine::json::ParseResult<Self> {
        i128::from_value(value).map(Density)
    }
}
