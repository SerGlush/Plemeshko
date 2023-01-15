macro_rules! declare_unit {
    ($name:ident, $ty:ty) => {
        #[derive(
            Clone,
            Copy,
            derive_more::FromStr,
            Default,
            Debug,
            derive_more::Display,
            serde::Serialize,
            serde::Deserialize,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            derive_more::Add,
            derive_more::Sub,
            derive_more::AddAssign,
            derive_more::SubAssign,
            derive_more::Neg,
        )]
        pub struct $name(pub $ty);

        impl std::ops::Mul<$ty> for $name {
            type Output = $name;

            fn mul(self, rhs: $ty) -> Self::Output {
                $name(self.0 * rhs)
            }
        }

        impl std::ops::MulAssign<$ty> for $name {
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

macro_rules! impl_binop_unit {
    ($trait:ident, $this:ty, $other:ty, $out:ty, $out_con:expr, $f:ident) => {
        impl std::ops::$trait<$other> for $this {
            type Output = $out;

            fn $f(self, rhs: $other) -> Self::Output {
                $out_con(std::ops::$trait::$f(self.0, rhs.0))
            }
        }
    };
}

mod time;

pub use time::*;

declare_unit!(ResourceWeight, i64);
declare_unit!(ResourceAmount, i64);

impl_binop_unit!(
    Mul,
    ResourceAmount,
    ResourceWeight,
    ResourceWeight,
    ResourceWeight,
    mul
);
impl_binop_unit!(
    Mul,
    ResourceWeight,
    ResourceAmount,
    ResourceWeight,
    ResourceWeight,
    mul
);
