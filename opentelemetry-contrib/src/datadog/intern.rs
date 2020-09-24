use indexmap::set::IndexSet;

pub(crate) struct StringInterner {
    data: IndexSet<String>,
}

impl StringInterner {
    pub(crate) fn new() -> StringInterner {
        StringInterner {
            data: Default::default(),
        }
    }

    pub(crate) fn intern(&mut self, data: &str) -> u32 {
        if let Some(idx) = self.data.get_index_of(data) {
            return idx as u32;
        }
        self.data.insert_full(data.to_string()).0 as u32
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &String> {
        self.data.iter()
    }

    pub(crate) fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern() {
        let a = "a".to_string();
        let b = "b";
        let c = "c";

        let mut intern = StringInterner::new();
        let a_idx = intern.intern(a.as_str());
        let b_idx = intern.intern(b);
        let c_idx = intern.intern(c);
        let d_idx = intern.intern(a.as_str());
        let e_idx = intern.intern(c);

        assert_eq!(a_idx, 0);
        assert_eq!(b_idx, 1);
        assert_eq!(c_idx, 2);
        assert_eq!(d_idx, a_idx);
        assert_eq!(e_idx, c_idx);
    }
}
