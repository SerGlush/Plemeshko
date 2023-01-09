use std::ops::{Mul, MulAssign};

use derive_more::{Add, AddAssign, Display, FromStr, Neg, Sub, SubAssign};
use plegine::json::FromValue;
use plegine_derive::FromValue;

macro_rules! declare_amount_type {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            FromValue,
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
        pub struct $name(pub i128);

        impl Mul<i128> for $name {
            type Output = $name;

            fn mul(self, rhs: i128) -> Self::Output {
                $name(self.0 * rhs)
            }
        }

        impl MulAssign<i128> for $name {
            fn mul_assign(&mut self, rhs: i128) {
                self.0 *= rhs
            }
        }

        impl std::ops::Div for $name {
            type Output = i128;

            fn div(self, rhs: Self) -> i128 {
                self.0 / rhs.0
            }
        }
    };
}

declare_amount_type!(ResourceWeight);
declare_amount_type!(ResourceAmount);
declare_amount_type!(TransportAmount);

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
