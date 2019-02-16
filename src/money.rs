use num::BigRational;
use num::ToPrimitive;

pub type Money = BigRational;

pub struct WrappedMoney(Money);

pub fn zero() -> Money {
    Money::new(0.into(), 1.into())
}

pub fn from<T: Into<WrappedMoney>>(x: T) -> Money {
    let wrapped: WrappedMoney = x.into();
    wrapped.0
}

pub fn to_float(m: &Money) -> f64 {
    m.numer().to_f64().unwrap() / m.denom().to_f64().unwrap()
}

pub fn parse(s: &str) -> Money {
    match s.parse::<f64>() {
        Ok(f) => BigRational::from_float(f).unwrap(),
        _ => s.parse().unwrap(),
    }
}

impl From<i32> for WrappedMoney {
    fn from(x: i32) -> WrappedMoney {
        WrappedMoney(Money::from_integer(x.into()))
    }
}

impl From<usize> for WrappedMoney {
    fn from(x: usize) -> WrappedMoney {
        WrappedMoney(Money::from_integer(x.into()))
    }
}
