//! Conversion of [`valuable::Value`] into OpenTelemetry [`AnyValue`].

use opentelemetry::{logs::AnyValue, Key};
use std::collections::HashMap;
use valuable::{Fields, NamedValues, Value, Visit};

/// Converts a [`valuable::Value`] into an [`AnyValue`], preserving its structure.
///
/// Returns `None` for [`Value::Unit`] (which also represents `Option::None`), as
/// `AnyValue` has no representation for an empty value. Such values are omitted.
pub(crate) fn to_any_value(value: Value<'_>) -> Option<AnyValue> {
    let any_value = match value {
        Value::Bool(v) => AnyValue::from(v),
        Value::Char(v) => AnyValue::from(v.to_string()),
        Value::F32(v) => AnyValue::from(f64::from(v)),
        Value::F64(v) => AnyValue::from(v),
        Value::I8(v) => AnyValue::from(i64::from(v)),
        Value::I16(v) => AnyValue::from(i64::from(v)),
        Value::I32(v) => AnyValue::from(i64::from(v)),
        Value::I64(v) => AnyValue::from(v),
        Value::I128(v) => int_or_string(v),
        Value::Isize(v) => int_or_string(v),
        Value::U8(v) => AnyValue::from(i64::from(v)),
        Value::U16(v) => AnyValue::from(i64::from(v)),
        Value::U32(v) => AnyValue::from(i64::from(v)),
        Value::U64(v) => int_or_string(v),
        Value::U128(v) => int_or_string(v),
        Value::Usize(v) => int_or_string(v),
        Value::String(v) => AnyValue::from(v.to_owned()),
        Value::Path(v) => AnyValue::from(v.display().to_string()),
        Value::Error(v) => AnyValue::from(v.to_string()),
        Value::Unit => return None,
        Value::Listable(v) => {
            let mut visitor = ListVisitor(Vec::with_capacity(v.size_hint().0));
            v.visit(&mut visitor);
            visitor.into_any_value()
        }
        Value::Tuplable(v) => {
            let mut visitor = ListVisitor(Vec::new());
            v.visit(&mut visitor);
            visitor.into_any_value()
        }
        Value::Mappable(v) => {
            let mut visitor = MapVisitor(HashMap::with_capacity(v.size_hint().0));
            v.visit(&mut visitor);
            visitor.into_any_value()
        }
        Value::Structable(v) => {
            fields_to_any_value(v.definition().fields(), |visitor| v.visit(visitor))?
        }
        Value::Enumerable(v) => {
            let variant = v.variant();
            let name = variant.name();
            match variant.fields() {
                // Unit variants are represented by their name.
                Fields::Unnamed(0) => AnyValue::from(name.to_owned()),
                // Other variants are represented as `{ variant_name: fields }`.
                fields => {
                    let inner = fields_to_any_value(fields, |visitor| v.visit(visitor))?;
                    AnyValue::Map(Box::new(HashMap::from([(
                        Key::new(name.to_owned()),
                        inner,
                    )])))
                }
            }
        }
        // `Value` is non-exhaustive; fall back to `Debug` formatting for future variants.
        _ => AnyValue::from(format!("{value:?}")),
    };
    Some(any_value)
}

fn int_or_string<T>(value: T) -> AnyValue
where
    T: TryInto<i64> + ToString + Copy,
{
    match value.try_into() {
        Ok(signed) => AnyValue::from(signed),
        Err(_) => AnyValue::from(value.to_string()),
    }
}

/// Converts the fields of a struct or enum variant. Named fields become a `Map`,
/// a single unnamed field (newtype) is unwrapped and other unnamed fields become a `ListAny`.
fn fields_to_any_value(
    fields: &Fields<'_>,
    visit: impl FnOnce(&mut dyn Visit),
) -> Option<AnyValue> {
    match fields {
        Fields::Named(named) => {
            let mut visitor = MapVisitor(HashMap::with_capacity(named.len()));
            visit(&mut visitor);
            Some(visitor.into_any_value())
        }
        Fields::Unnamed(1) => {
            let mut visitor = ListVisitor(Vec::with_capacity(1));
            visit(&mut visitor);
            visitor.0.pop()
        }
        Fields::Unnamed(len) => {
            let mut visitor = ListVisitor(Vec::with_capacity(*len));
            visit(&mut visitor);
            Some(visitor.into_any_value())
        }
    }
}

struct ListVisitor(Vec<AnyValue>);

impl ListVisitor {
    fn into_any_value(self) -> AnyValue {
        AnyValue::ListAny(Box::new(self.0))
    }
}

impl Visit for ListVisitor {
    fn visit_value(&mut self, value: Value<'_>) {
        self.0.extend(to_any_value(value));
    }

    /// NOTE: `AnyValue` does not have an equivalent to represent `None`,
    /// this matches appender-log thus empty values will be filtered,
    /// changing order in anonymous structs and reducing size of lists
    fn visit_unnamed_fields(&mut self, values: &[Value<'_>]) {
        self.0
            .extend(values.iter().copied().filter_map(to_any_value));
    }
}

struct MapVisitor(HashMap<Key, AnyValue>);

impl MapVisitor {
    fn into_any_value(self) -> AnyValue {
        AnyValue::Map(Box::new(self.0))
    }
}

impl Visit for MapVisitor {
    fn visit_value(&mut self, _value: Value<'_>) {}

    fn visit_named_fields(&mut self, named_values: &NamedValues<'_>) {
        for (field, value) in named_values {
            if let Some(value) = to_any_value(*value) {
                self.0.insert(Key::new(field.name().to_owned()), value);
            }
        }
    }

    fn visit_entry(&mut self, key: Value<'_>, value: Value<'_>) {
        let key = match key {
            Value::String(key) => key.to_owned(),
            key => format!("{key:?}"),
        };
        if let Some(value) = to_any_value(value) {
            self.0.insert(Key::new(key), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::to_any_value;
    use opentelemetry::{logs::AnyValue, Key};
    use std::collections::HashMap;
    use valuable::Valuable;

    #[derive(Valuable)]
    struct User {
        name: &'static str,
        age: u8,
        email: Option<&'static str>,
        roles: Vec<Role>,
        address: Address,
    }

    #[derive(Valuable)]
    struct Address(&'static str, u16);

    #[derive(Valuable)]
    enum Role {
        Admin,
        Member { team: &'static str },
        Guest(u64),
    }

    #[derive(Valuable)]
    struct Node {
        value: i64,
        child: Option<Box<Node>>,
    }

    fn map<const N: usize>(entries: [(&'static str, AnyValue); N]) -> AnyValue {
        AnyValue::Map(Box::new(
            entries
                .into_iter()
                .map(|(k, v)| (Key::from_static_str(k), v))
                .collect(),
        ))
    }

    #[test]
    fn primitives() {
        assert_eq!(to_any_value(true.as_value()), Some(AnyValue::Boolean(true)));
        assert_eq!(to_any_value('x'.as_value()), Some(AnyValue::from("x")));
        assert_eq!(to_any_value(1.5f32.as_value()), Some(AnyValue::Double(1.5)));
        assert_eq!(to_any_value(42u8.as_value()), Some(AnyValue::Int(42)));
        assert_eq!(to_any_value((-42i32).as_value()), Some(AnyValue::Int(-42)));
        assert_eq!(
            to_any_value(u64::MAX.as_value()),
            Some(AnyValue::from(u64::MAX.to_string()))
        );
        assert_eq!(
            to_any_value("otel".as_value()),
            Some(AnyValue::from("otel"))
        );
        assert_eq!(to_any_value(valuable::Value::Unit), None);
        assert_eq!(to_any_value(None::<u8>.as_value()), None);
    }

    #[test]
    fn collections() {
        assert_eq!(
            to_any_value(vec![1i64, 2, 3].as_value()),
            Some(AnyValue::ListAny(Box::new(vec![
                AnyValue::Int(1),
                AnyValue::Int(2),
                AnyValue::Int(3),
            ])))
        );
        assert_eq!(
            to_any_value(("a", 1i64).as_value()),
            Some(AnyValue::ListAny(Box::new(vec![
                AnyValue::from("a"),
                AnyValue::Int(1),
            ])))
        );
        let entries = HashMap::from([("key", 1i64)]);
        assert_eq!(
            to_any_value(entries.as_value()),
            Some(map([("key", AnyValue::Int(1))]))
        );
    }

    #[test]
    fn structs_and_enums() {
        let user = User {
            name: "otel",
            age: 7,
            email: None,
            roles: vec![Role::Admin, Role::Member { team: "rust" }, Role::Guest(1)],
            address: Address("main street", 1),
        };
        let expected = map([
            ("name", AnyValue::from("otel")),
            ("age", AnyValue::Int(7)),
            (
                "roles",
                AnyValue::ListAny(Box::new(vec![
                    AnyValue::from("Admin"),
                    map([("Member", map([("team", AnyValue::from("rust"))]))]),
                    map([("Guest", AnyValue::Int(1))]),
                ])),
            ),
            (
                "address",
                AnyValue::ListAny(Box::new(vec![
                    AnyValue::from("main street"),
                    AnyValue::Int(1),
                ])),
            ),
        ]);
        assert_eq!(to_any_value(user.as_value()), Some(expected));
    }

    #[test]
    fn nested() {
        let node = (1..=64)
            .rev()
            .fold(None, |child, value| Some(Box::new(Node { value, child })));
        let expected = (1..=64).rev().fold(None, |child, value| {
            let value = ("value", AnyValue::Int(value));
            Some(match child {
                Some(child) => map([value, ("child", child)]),
                None => map([value]),
            })
        });
        assert_eq!(to_any_value(node.as_value()), expected);
    }
}
