#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::KeyValue;
    use std::hash::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn equality_kv_float() {
        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.0);
        assert_eq!(kv1, kv2);

        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.01);
        assert_ne!(kv1, kv2);

        let kv1 = KeyValue::new("key", std::f64::NAN);
        let kv2 = KeyValue::new("key", std::f64::NAN);
        assert_ne!(kv1, kv2, "NAN is not equal to itself");

        let kv1 = KeyValue::new("key", std::f64::INFINITY);
        let kv2 = KeyValue::new("key", std::f64::INFINITY);
        assert_eq!(kv1, kv2);

        let kv1 = KeyValue::new("key", std::f64::NEG_INFINITY);
        let kv2 = KeyValue::new("key", std::f64::NEG_INFINITY);
        assert_eq!(kv1, kv2);

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(kv1, kv2);
        }
    }

    #[test]
    fn hash_kv_float() {
        let kv1 = KeyValue::new("key", 1.0);
        let kv2 = KeyValue::new("key", 1.0);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", 1.001);
        let kv2 = KeyValue::new("key", 1.001);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", 1.001);
        let kv2 = KeyValue::new("key", 1.002);
        assert_ne!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", std::f64::NAN);
        let kv2 = KeyValue::new("key", std::f64::NAN);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let kv1 = KeyValue::new("key", std::f64::INFINITY);
        let kv2 = KeyValue::new("key", std::f64::INFINITY);
        assert_eq!(hash_helper(&kv1), hash_helper(&kv2));

        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let random_value = rng.gen::<f64>();
            let kv1 = KeyValue::new("key", random_value);
            let kv2 = KeyValue::new("key", random_value);
            assert_eq!(hash_helper(&kv1), hash_helper(&kv2));
        }
    }

    #[test]
    fn hash_kv_order() {
        let float_vals = [
            0.0,
            1.0,
            -1.0,
            std::f64::INFINITY,
            std::f64::NEG_INFINITY,
            std::f64::NAN,
            std::f64::MIN,
            std::f64::MAX,
        ];

        for v in float_vals {
            let kv1 = KeyValue::new("a", v);
            let kv2 = KeyValue::new("b", v);
            assert!(kv1 < kv2, "Order is solely based on key!");
        }
    }

    fn hash_helper<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }
}
