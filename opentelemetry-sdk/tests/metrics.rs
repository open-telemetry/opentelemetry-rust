#[cfg(test)]
#[cfg(feature = "metrics")]
mod metrics {
    use std::time::Duration;

    use opentelemetry_api::metrics::{Counter, MeterProvider, UpDownCounter};
    use opentelemetry_api::Context;
    use opentelemetry_sdk::export::metrics::aggregation::{
        cumulative_temporality_selector, delta_temporality_selector, LastValue, Sum,
        TemporalitySelector,
    };
    use opentelemetry_sdk::export::metrics::InstrumentationLibraryReader;
    use opentelemetry_sdk::metrics::aggregators::{LastValueAggregator, SumAggregator};
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
            context: Context,

            gauge_tx: crossbeam_channel::Sender<i64>,
            counter: Counter<f64>,
            up_down_counter: UpDownCounter<f64>,

            results: Vec<Vec<(String, f64)>>,
        }

        impl<T, F> TestSuite<T, F>
        where
            F: Fn() -> T,
            T: TemporalitySelector + Clone + Send + Sync + 'static,
        {
            fn setup(f: F) -> Self {
                let controller = controllers::basic(processors::factory(
                    selectors::simple::inexpensive(), // basically give us Sum aggregation except for gauge, which is LastValue aggregation
                    f(),
                ))
                .with_collect_period(Duration::ZERO) // require manual collection
                .build();
                let meter = controller.versioned_meter("test", None, None);
                let (gauge_tx, gauge_rx) = crossbeam_channel::bounded(10);
                let gauge = meter.i64_observable_gauge("gauge").init();
                meter
                    .register_callback(move |cx| {
                        if let Ok(val) = gauge_rx.try_recv() {
                            gauge.observe(cx, val, &[]);
                        }
                    })
                    .expect("failed to register callback");
                TestSuite {
                    controller,
                    temporality: f,
                    counter: meter.f64_counter("counter").init(),
                    up_down_counter: meter.f64_up_down_counter("up_down_counter").init(),
                    context: Context::new(),
                    gauge_tx,
                    results: Vec::new(),
                }
            }

            fn add_counter(&mut self, val: f64) {
                self.counter.add(&self.context, val, &[]);
            }

            fn change_up_down_counter(&mut self, val: f64) {
                self.up_down_counter.add(&self.context, val, &[]);
            }

            fn change_gauge(&mut self, val: i64) {
                self.gauge_tx.send(val).unwrap();
            }

            fn collect_and_save(&mut self) {
                self.controller.collect(&self.context).unwrap();

                let temporality = (self.temporality)();
                let mut result_per_round = Vec::new();
                self.controller
                    .try_for_each(&mut |_library, reader| {
                        reader.try_for_each(&temporality, &mut |record| {
                            if let Some(sum_agg) = record
                                .aggregator()
                                .unwrap()
                                .as_any()
                                .downcast_ref::<SumAggregator>()
                            {
                                result_per_round.push((
                                    record.descriptor().name().to_owned(),
                                    sum_agg.sum().unwrap().to_f64(&NumberKind::F64),
                                ));
                            }

                            if let Some(last_value_agg) = record
                                .aggregator()
                                .unwrap()
                                .as_any()
                                .downcast_ref::<LastValueAggregator>()
                            {
                                result_per_round.push((
                                    record.descriptor().name().to_owned(),
                                    last_value_agg
                                        .last_value()
                                        .unwrap()
                                        .0
                                        .to_f64(&NumberKind::I64),
                                ))
                            }

                            Ok(())
                        })?;
                        Ok(())
                    })
                    .expect("no error expected");
                // sort result per round stablely so we have deterministic results
                result_per_round.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
                self.results.push(result_per_round);
            }

            fn expect(self, expected: Vec<Vec<(impl Into<String>, f64)>>) {
                assert_eq!(
                    self.results,
                    expected
                        .into_iter()
                        .map(|v| v
                            .into_iter()
                            .map(|(k, v)| (k.into(), v))
                            .collect::<Vec<_>>())
                        .collect::<Vec<_>>()
                );
            }
        }

        let mut cumulative = TestSuite::setup(cumulative_temporality_selector);
        // round 1
        cumulative.add_counter(10.0);
        cumulative.add_counter(5.3);
        cumulative.collect_and_save();
        // round 2
        cumulative.collect_and_save();
        // round 3
        cumulative.add_counter(0.0);
        cumulative.change_up_down_counter(-1.0);
        cumulative.change_gauge(-1);
        cumulative.collect_and_save();
        // round 4
        cumulative.change_up_down_counter(1.0);
        cumulative.add_counter(10.0);
        cumulative.collect_and_save();
        // round 5
        cumulative.change_gauge(1);
        cumulative.collect_and_save();
        // assert
        cumulative.expect(vec![
            vec![("counter", 15.3)],
            vec![("counter", 15.3)],
            vec![
                ("counter", 15.3),
                ("gauge", -1.0),
                ("up_down_counter", -1.0),
            ],
            vec![("counter", 25.3), ("gauge", -1.0), ("up_down_counter", 0.0)],
            vec![("counter", 25.3), ("gauge", 1.0), ("up_down_counter", 0.0)],
        ]);

        let mut delta = TestSuite::setup(delta_temporality_selector);
        // round 1
        delta.add_counter(10.0);
        delta.add_counter(5.3);
        delta.collect_and_save();
        // round 2
        delta.collect_and_save();
        // round 3
        delta.add_counter(10.0);
        delta.collect_and_save();
        // round 4
        delta.add_counter(0.0);
        delta.collect_and_save();
        // round 5
        delta.change_up_down_counter(-1.0);
        delta.change_gauge(-1);
        delta.collect_and_save();
        // round 6
        delta.change_up_down_counter(1.0);
        delta.collect_and_save();
        // assert
        delta.expect(vec![
            vec![("counter", 15.3)],
            vec![], // no change and no data exported
            vec![("counter", 10.0)],
            vec![("counter", 0.0)],
            vec![("gauge", -1.0), ("up_down_counter", -1.0)],
            vec![("up_down_counter", 1.0)],
        ])
    }
}
