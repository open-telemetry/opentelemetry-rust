use opentelemetry::time::now;
use std::time::SystemTime;
// leaky bucket based rate limit
// should be Send+Sync
pub(crate) struct LeakyBucket {
    span_per_sec: f64,
    available: f64,
    bucket_size: f64,
    last_time: SystemTime,
}
use opentelemetry::otel_debug;

fn sanitize_non_negative(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

impl LeakyBucket {
    pub(crate) fn new(bucket_size: f64, span_per_sec: f64) -> LeakyBucket {
        let bucket_size = sanitize_non_negative(bucket_size);
        let span_per_sec = sanitize_non_negative(span_per_sec);
        LeakyBucket {
            span_per_sec,
            available: bucket_size,
            bucket_size,
            last_time: now(),
        }
    }

    pub(crate) fn update(&mut self, span_per_sec: f64) {
        self.span_per_sec = sanitize_non_negative(span_per_sec);
    }

    pub(crate) fn should_sample(&mut self) -> bool {
        self.check_availability(now)
    }

    fn check_availability<F>(&mut self, now: F) -> bool
    where
        F: Fn() -> SystemTime,
    {
        let cur_time = now();
        match cur_time.duration_since(self.last_time) {
            Ok(dur) => {
                self.last_time = cur_time;
                self.available = f64::min(
                    dur.as_secs_f64() * self.span_per_sec + self.available,
                    self.bucket_size,
                );

                if self.available >= 1.0 {
                    self.available -= 1.0;
                    true
                } else {
                    false
                }
            }
            Err(err) => {
                otel_debug!(
                    name: "JaegerRemoteSampler.LeakyBucket.ClockAdjustment",
                    message = "Jaeger remote sampler detected a rewind in system clock",
                    reason = format!("{:?}", err),
                );
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::trace::sampler::jaeger_remote::rate_limit::LeakyBucket;
    use opentelemetry::time::now;
    use std::ops::{Add, Sub};
    use std::time::Duration;

    #[test]
    fn test_leaky_bucket() {
        // maximum bucket size 2, add 1 allowance every 10 seconds
        let mut leaky_bucket = LeakyBucket::new(2.0, 0.1);
        let current_time = now();
        leaky_bucket.last_time = current_time;

        let test_cases = vec![
            (0, vec![true, true, false]),
            (1, vec![false]),
            (5, vec![false]),
            (10, vec![true, false]),
            (60, vec![true, true, false]), // maximum allowance is 2
        ];

        for (elapsed_sec, cases) in test_cases.into_iter() {
            for should_pass in cases {
                assert_eq!(
                    should_pass,
                    leaky_bucket.check_availability(|| {
                        current_time.add(Duration::from_secs(elapsed_sec))
                    })
                )
            }
        }
    }

    #[test]
    fn test_rewind_clock_should_pass() {
        let mut leaky_bucket = LeakyBucket::new(2.0, 0.1);
        let current_time = now();
        leaky_bucket.last_time = current_time;

        assert!(leaky_bucket.check_availability(|| { current_time.sub(Duration::from_secs(10)) }))
    }

    #[test]
    fn test_leaky_bucket_subsecond_refill() {
        let mut leaky_bucket = LeakyBucket::new(1.0, 10.0);
        let t0 = now();
        leaky_bucket.last_time = t0;

        assert!(leaky_bucket.check_availability(|| t0));
        assert!(!leaky_bucket.check_availability(|| t0));

        for ms in [100, 200, 300] {
            let t = t0.add(Duration::from_millis(ms));
            assert!(
                leaky_bucket.check_availability(|| t),
                "expected 1 token refilled after {ms}ms at 10 spans/sec"
            );
        }

        // Sub-second partial progress must accumulate across checks rather than reset last_time.
        assert!(!leaky_bucket.check_availability(|| t0.add(Duration::from_millis(350))));
        assert!(leaky_bucket.check_availability(|| t0.add(Duration::from_millis(400))));
        assert!(!leaky_bucket.check_availability(|| t0.add(Duration::from_millis(450))));
        assert!(leaky_bucket.check_availability(|| t0.add(Duration::from_millis(500))));
    }

    #[test]
    fn test_leaky_bucket_idle_does_not_double_burst() {
        let mut leaky_bucket = LeakyBucket::new(2.0, 1.0);
        let t0 = now();
        leaky_bucket.last_time = t0;

        let t_idle = t0.add(Duration::from_secs(10));
        let results: Vec<bool> = (0..4)
            .map(|_| leaky_bucket.check_availability(|| t_idle))
            .collect();
        assert_eq!(results, vec![true, true, false, false]);
    }

    #[test]
    fn test_leaky_bucket_clamps_negative_or_non_finite_params() {
        let t0 = now();

        let mut neg_rate = LeakyBucket::new(1.0, -2.0);
        neg_rate.last_time = t0;
        assert!(neg_rate.check_availability(|| t0));
        assert!(!neg_rate.check_availability(|| t0.add(Duration::from_secs(10))));
        neg_rate.update(1.0);
        assert!(neg_rate.check_availability(|| t0.add(Duration::from_secs(11))));

        let mut neg_bucket = LeakyBucket::new(-1.0, 1.0);
        neg_bucket.last_time = t0;
        assert!(!neg_bucket.check_availability(|| t0));
        assert!(!neg_bucket.check_availability(|| t0.add(Duration::from_secs(10))));

        let mut non_finite = LeakyBucket::new(f64::NAN, f64::INFINITY);
        non_finite.last_time = t0;
        assert!(!non_finite.check_availability(|| t0.add(Duration::from_secs(1))));
        non_finite.update(f64::NAN);
        assert!(!non_finite.check_availability(|| t0.add(Duration::from_secs(2))));
    }
}
