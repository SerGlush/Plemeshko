use derive_more::{Add, AddAssign, Display, FromStr, Neg, Sub, SubAssign};
use serde::Deserialize;
use std::ops::{Mul, MulAssign};

macro_rules! declare_amount_type {
    ($name:ident, $ty:ty) => {
        #[derive(
            Clone,
            Copy,
            Deserialize,
            FromStr,
            Default,
            Display,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Add,
            Sub,
            AddAssign,
            SubAssign,
            Neg,
        )]
        pub struct $name(pub $ty);

        impl Mul<$ty> for $name {
            type Output = $name;

            fn mul(self, rhs: $ty) -> Self::Output {
                $name(self.0 * rhs)
            }
        }

        impl MulAssign<$ty> for $name {
            fn mul_assign(&mut self, rhs: $ty) {
                self.0 *= rhs
            }
        }

        impl std::ops::Div for $name {
            type Output = $ty;

            fn div(self, rhs: Self) -> $ty {
                self.0 / rhs.0
            }
        }
    };
}

declare_amount_type!(ResourceWeight, i64);
declare_amount_type!(ResourceAmount, i64);
declare_amount_type!(Ticks, i64);

impl Mul<ResourceWeight> for ResourceAmount {
    type Output = ResourceWeight;

    fn mul(self, rhs: ResourceWeight) -> Self::Output {
        ResourceWeight(self.0 * rhs.0)
    }
}

impl Mul<ResourceAmount> for ResourceWeight {
    type Output = ResourceWeight;

    fn mul(self, rhs: ResourceAmount) -> Self::Output {
        ResourceWeight(self.0 * rhs.0)
    }
}
