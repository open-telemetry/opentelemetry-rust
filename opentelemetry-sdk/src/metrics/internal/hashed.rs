use std::{
    borrow::{Borrow, Cow},
    hash::{BuildHasher, DefaultHasher, Hash, Hasher},
    ops::Deref,
};

/// Hash value only once, works with references and owned types.
pub(crate) struct Hashed<'a, T>
where
    T: ToOwned + ?Sized,
{
    value: Cow<'a, T>,
    hash: u64,
}

impl<'a, T> Hashed<'a, T>
where
    T: ToOwned + Hash + ?Sized,
{
    pub(crate) fn from_borrowed(value: &'a T) -> Self {
        let hash = calc_hash(&value);
        Self {
            value: Cow::Borrowed(value),
            hash,
        }
    }

    pub(crate) fn from_owned(value: <T as ToOwned>::Owned) -> Self {
        let hash = calc_hash(value.borrow());
        Self {
            value: Cow::Owned(value),
            hash,
        }
    }

    pub(crate) fn into_owned(self) -> Hashed<'static, T> {
        let value = self.value.into_owned();
        Hashed {
            value: Cow::Owned(value),
            hash: self.hash,
        }
    }

    pub(crate) fn into_inner_owned(self) -> T::Owned {
        self.value.into_owned()
    }
}

fn calc_hash<T>(value: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

impl<T> Clone for Hashed<'_, T>
where
    T: ToOwned + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            hash: self.hash,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.value.clone_from(&source.value);
        self.hash = source.hash;
    }
}

impl<T> Hash for Hashed<'_, T>
where
    T: ToOwned + Hash + ?Sized,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl<T> PartialEq for Hashed<'_, T>
where
    T: ToOwned + PartialEq + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.as_ref() == other.value.as_ref()
    }
}

impl<T> Eq for Hashed<'_, T> where T: ToOwned + Eq + ?Sized {}

impl<T> Deref for Hashed<'_, T>
where
    T: ToOwned + ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

/// Used to make [`Hashed`] values no-op in [`HashMap`](std::collections::HashMap) or [`HashSet`](std::collections::HashSet).
/// For all other keys types (except for [`u64`]) it will panic.
#[derive(Default, Clone)]
pub(crate) struct HashedNoOpBuilder {
    hashed: u64,
}

impl Hasher for HashedNoOpBuilder {
    fn finish(&self) -> u64 {
        self.hashed
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("Only works with `Hashed` value")
    }

    fn write_u64(&mut self, i: u64) {
        self.hashed = i;
    }
}

impl BuildHasher for HashedNoOpBuilder {
    type Hasher = HashedNoOpBuilder;

    fn build_hasher(&self) -> Self::Hasher {
        HashedNoOpBuilder::default()
    }
}
