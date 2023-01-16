#[cfg(test)]
#[cfg(feature = "metrics")]
mod metrics {
    use std::time::Duration;

    use opentelemetry_api::metrics::{Counter, MeterProvider};
    use opentelemetry_api::Context;
    use opentelemetry_sdk::export::metrics::aggregation::{
        cumulative_temporality_selector, delta_temporality_selector, Sum, TemporalitySelector,
    };
    use opentelemetry_sdk::export::metrics::InstrumentationLibraryReader;
    use opentelemetry_sdk::metrics::aggregators::SumAggregator;
    use opentelemetry_sdk::metrics::controllers::BasicController;
    use opentelemetry_sdk::metrics::sdk_api::NumberKind;
    use opentelemetry_sdk::metrics::{controllers, processors, selectors};

    #[test]
    fn test_temporality() {
        struct TestSuite<T, F>
        where
            T: TemporalitySelector + Clone + Send + Sync + 'static,
            F: Fn() -> T,
        {
            temporality: F,
            controller: BasicController,
            counter: Counter<f64>,
            context: Context,

            results: Vec<f64>,
        }

        impl<T, F> TestSuite<T, F>
        where
            F: Fn() -> T,
            T: TemporalitySelector + Clone + Send + Sync + 'static,
        {
            fn setup(f: F) -> Self {
                let controller = controllers::basic(processors::factory(
                    selectors::simple::inexpensive(), // basically give us SUM aggregation
                    f(),
                ))
                .with_collect_period(Duration::ZERO) // require manual collection
                .build();
                let meter = controller.versioned_meter("test", None, None);
                TestSuite {
                    controller,
                    temporality: f,
                    counter: meter.f64_counter("counter").init(),
                    context: Context::new(),
                    results: Vec::new(),
                }
            }

            fn add(&mut self, val: f64) {
                self.counter.add(&self.context, val, &[]);
            }

            fn collect_and_save(&mut self) {
                self.controller.collect(&self.context).unwrap();

                let temporality = (self.temporality)();
                self.controller
                    .try_for_each(&mut |_library, reader| {
                        reader.try_for_each(&temporality, &mut |record| {
                            let agg = record
                                .aggregator()
                                .unwrap()
                                .as_any()
                                .downcast_ref::<SumAggregator>()
                                .unwrap();
                            self.results
                                .push(agg.sum().unwrap().to_f64(&NumberKind::F64));
                            Ok(())
                        })?;
                        Ok(())
                    })
                    .expect("no error expected");
            }

            fn expect(self, expected: Vec<f64>) {
                assert_eq!(self.results, expected)
            }
        }

        let mut cumulative = TestSuite::setup(cumulative_temporality_selector);
        cumulative.add(10.0);
        cumulative.add(5.3);
        cumulative.collect_and_save();

        cumulative.collect_and_save();
        cumulative.add(10.0);
        cumulative.collect_and_save();
        cumulative.expect(vec![15.3, 15.3, 25.3]);

        let mut delta = TestSuite::setup(delta_temporality_selector);
        delta.add(10.0);
        delta.add(5.3);
        delta.collect_and_save();
        delta.collect_and_save();
        delta.add(10.0);
        delta.collect_and_save();
        delta.expect(vec![15.3, 10.0])
    }
}
