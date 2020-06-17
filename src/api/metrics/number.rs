use std::cmp;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Number represents either an integral or a floating point value. It
/// needs to be accompanied with a source of NumberKind that describes
/// the actual type of the value stored within Number.
#[derive(Debug, Default)]
pub struct Number(AtomicU64);

impl Number {
    /// Assigns this number to the given other number. Both should be of the same kind.
    pub fn assign(&self, number_kind: &NumberKind, other: &Number) {
        let other = other.0.load(Ordering::Acquire);
        match number_kind {
            NumberKind::F64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let new = u64_to_f64(other);
                let swapped = self
                    .0
                    .compare_and_swap(current, f64_to_u64(new), Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::U64 | NumberKind::I64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let swapped = self.0.compare_and_swap(current, other, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
        }
    }

    /// Adds this number to the given other number. Both should be of the same kind.
    pub fn saturating_add(&self, number_kind: &NumberKind, other: &Number) {
        match number_kind {
            NumberKind::I64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let other = other.0.load(Ordering::Acquire);
                let new = (current as i64).saturating_add(other as i64) as u64;
                let swapped = self.0.compare_and_swap(current, new, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::F64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let other = other.0.load(Ordering::Acquire);
                let new = u64_to_f64(current) + u64_to_f64(other);
                let swapped = self
                    .0
                    .compare_and_swap(current, f64_to_u64(new), Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::U64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let other = other.0.load(Ordering::Acquire);
                let new = current.saturating_add(other);
                let swapped = self.0.compare_and_swap(current, new, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
        }
    }

    /// Casts the number to `i64`. May result in data/precision loss.
    pub fn to_i64(&self, number_kind: &NumberKind) -> i64 {
        let current = self.0.load(Ordering::SeqCst);

        match number_kind {
            NumberKind::F64 => u64_to_f64(current) as i64,
            NumberKind::U64 | NumberKind::I64 => current as i64,
        }
    }

    /// Casts the number to `u64`. May result in data/precision loss.
    pub fn to_u64(&self, number_kind: &NumberKind) -> u64 {
        let current = self.0.load(Ordering::SeqCst);

        match number_kind {
            NumberKind::F64 => u64_to_f64(current) as u64,
            NumberKind::U64 | NumberKind::I64 => current,
        }
    }

    /// Casts the number to `f64`. May result in data/precision loss.
    pub fn to_f64(&self, number_kind: &NumberKind) -> f64 {
        let current = self.0.load(Ordering::SeqCst);

        match number_kind {
            NumberKind::I64 => (current as i64) as f64,
            NumberKind::F64 => u64_to_f64(current),
            NumberKind::U64 => current as f64,
        }
    }

    /// Compares this number to the given other number. Both should be of the same kind.
    pub fn partial_cmp(&self, number_kind: &NumberKind, other: &Number) -> Option<cmp::Ordering> {
        let current = self.0.load(Ordering::SeqCst);
        let other = other.0.load(Ordering::SeqCst);
        match number_kind {
            NumberKind::I64 => (current as i64).partial_cmp(&(other as i64)),
            NumberKind::F64 => {
                let current = u64_to_f64(current);
                let other = u64_to_f64(other);
                current.partial_cmp(&other)
            }
            NumberKind::U64 => current.partial_cmp(&other),
        }
    }

    /// Checks if this value ia an f64 nan value. Do not use on non-f64 values.
    pub fn is_nan(&self) -> bool {
        let current = self.0.load(Ordering::Acquire);
        u64_to_f64(current).is_nan()
    }

    /// `true` if the actual value is less than zero.
    pub fn is_negative(&self, number_kind: &NumberKind) -> bool {
        match number_kind {
            NumberKind::I64 => {
                let current = self.0.load(Ordering::Acquire);
                (current as i64).is_negative()
            }
            NumberKind::F64 => {
                let current = self.0.load(Ordering::Acquire);
                u64_to_f64(current).is_sign_negative()
            }
            NumberKind::U64 => false,
        }
    }

    /// Return loaded data for debugging purposes
    pub fn to_debug(&self, kind: &NumberKind) -> Box<dyn fmt::Debug> {
        let current = self.0.load(Ordering::SeqCst);
        match kind {
            NumberKind::I64 => Box::new(current as i64),
            NumberKind::F64 => Box::new(u64_to_f64(current)),
            NumberKind::U64 => Box::new(current),
        }
    }
}

impl Clone for Number {
    fn clone(&self) -> Self {
        self.0.load(Ordering::SeqCst).into()
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number(AtomicU64::new(f64_to_u64(f)))
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Number(AtomicU64::new(i as u64))
    }
}

impl From<u64> for Number {
    fn from(u: u64) -> Self {
        Number(AtomicU64::new(u))
    }
}

/// A descriptor for the encoded data type of a `Number`
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum NumberKind {
    /// An Number that stores `i64` values.
    I64,
    /// An Number that stores `f64` values.
    F64,
    /// An Number that stores `u64` values.
    U64,
}

impl NumberKind {
    /// Returns the zero value for each kind
    pub fn zero(&self) -> Number {
        match self {
            NumberKind::I64 => 0i64.into(),
            NumberKind::F64 => 0f64.into(),
            NumberKind::U64 => 0u64.into(),
        }
    }

    /// Returns the max value for each kind
    pub fn max(&self) -> Number {
        match self {
            NumberKind::I64 => i64::MAX.into(),
            NumberKind::F64 => f64::MAX.into(),
            NumberKind::U64 => u64::MAX.into(),
        }
    }

    /// Returns the min value for each kind
    pub fn min(&self) -> Number {
        match self {
            NumberKind::I64 => i64::MIN.into(),
            NumberKind::F64 => f64::MIN.into(),
            NumberKind::U64 => u64::MIN.into(),
        }
    }
}

#[inline]
fn u64_to_f64(val: u64) -> f64 {
    f64::from_bits(val)
}

#[inline]
fn f64_to_u64(val: f64) -> u64 {
    f64::to_bits(val)
}
