#[cfg(all(feature = "metrics", feature = "rt-tokio"))]
mod test {
    use http::header::{HeaderValue, AUTHORIZATION, USER_AGENT};
    use hyper::{
        body,
        service::{make_service_fn, service_fn},
        Body, Method, Request, Response, Server,
    };
    use opentelemetry::{global, Key, KeyValue};
    use std::net::SocketAddr;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread")]
    async fn integration_test() {
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
        let (req_tx, mut req_rx) = tokio::sync::mpsc::channel(1);
        let (tick_tx, tick_rx) = tokio::sync::watch::channel(0);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let addr: SocketAddr = "[::1]:0".parse().unwrap();

        let server_handle = tokio::spawn(async move {
            let make_svc = make_service_fn(move |_| {
                let req_tx = req_tx.clone();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let req_tx = req_tx.clone();
                        async move {
                            if req.method() == Method::POST && req.uri().path() == "/test/a/b/c" {
                                req_tx.send(req).await.unwrap();
                                Ok::<_, hyper::Error>(Response::new(Body::empty()))
                            } else {
                                req_tx.send(req).await.unwrap();
                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(http::StatusCode::METHOD_NOT_ALLOWED)
                                        .body(Body::empty())
                                        .unwrap(),
                                )
                            }
                        }
                    }))
                }
            });

            let server = Server::bind(&addr).http1_only(true).serve(make_svc);

            addr_tx.send(server.local_addr()).unwrap();

            println!(
                "Starting http server on port {}",
                server.local_addr().port()
            );
            if let Err(err) = server
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
            {
                panic!("failed to start http server, {:?}", err);
            }
        });

        let addr = addr_rx.await.unwrap();

        let _meter = opentelemetry_dynatrace::new_pipeline()
            .metrics(tokio::spawn, move |_: Duration| {
                let mut tick_rx = tick_rx.clone();
                futures::stream::once(async move {
                    let _ = tick_rx.changed().await.is_ok();
                })
            })
            .with_exporter(opentelemetry_dynatrace::new_exporter().with_export_config(
                opentelemetry_dynatrace::ExportConfig {
                    endpoint: Some(format!("http://{}/test/a/b/c", addr)),
                    token: Some("1234567890".to_string()),
                },
            ))
            .with_prefix("example".to_string())
            .with_period(Duration::from_millis(100))
            .with_timestamp(false)
            .build()
            .unwrap();

        let (req, _) = tokio::join!(req_rx.recv(), async move {
            let meter = global::meter("ex.com/basic");

            let recorder = meter.u64_counter("test1").init();
            recorder.add(
                90,
                &[
                    KeyValue::new("A", "test1"),
                    KeyValue::new("B", "test2"),
                    KeyValue::new("C", "test3"),
                ],
            );

            let recorder = meter.f64_counter("test2").init();
            recorder.add(1e10 + 0.123, &[KeyValue::new("foo", "bar")]);

            let recorder = meter.i64_histogram("test3").init();
            recorder.record(-999, &[Key::new("foo").i64(-123)]);

            let _ = tick_tx.send(1);
        });

        assert!(req.is_some());

        let req = req.unwrap();

        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.uri().path(), "/test/a/b/c");
        assert_eq!(
            req.headers().get(USER_AGENT),
            Some(&HeaderValue::from_static("opentelemetry-metric-rust")),
        );
        assert_eq!(
            req.headers().get(AUTHORIZATION),
            Some(&HeaderValue::from_str("Api-Token 1234567890").unwrap()),
        );

        let bytes = body::to_bytes(req.into_body())
            .await
            .expect("http server body not readable");
        let body = String::from_utf8(bytes.to_vec()).expect("response is not valid utf-8");

        // We're done with this test request, so shut down the server.
        shutdown_tx
            .send(())
            .expect("sender error while shutting down http server");

        // Reap the task handle to ensure that the server did indeed shut down.
        let _ = server_handle.await.expect("http server yielded an error");

        let mut metric_lines: Vec<&str> = body.lines().collect();
        metric_lines.sort_unstable();

        let mut iter = metric_lines.iter();

        assert_eq!(
            Some(&"example.test1,a=test1,b=test2,c=test3,dt.metrics.source=opentelemetry gauge,90"),
            iter.next(),
        );
        assert_eq!(
            Some(&"example.test2,dt.metrics.source=opentelemetry,foo=bar gauge,10000000000.123"),
            iter.next(),
        );
        assert_eq!(
            Some(&"example.test3,dt.metrics.source=opentelemetry,foo=-123 gauge,-999"),
            iter.next(),
        );
        assert_eq!(iter.next(), None);
    }
}
