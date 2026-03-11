window.BENCHMARK_DATA = {
  "lastUpdate": 1773256249335,
  "repoUrl": "https://github.com/open-telemetry/opentelemetry-rust",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "Cathal",
            "username": "CathalMullan",
            "email": "contact@cathal.dev"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "d91b847f2b0382ac8ae72c08cb636b03873e62a3",
          "message": "chore(sdk): remove tokio runtime from testing feature (#3407)",
          "timestamp": "2026-03-10T14:30:17Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/d91b847f2b0382ac8ae72c08cb636b03873e62a3"
        },
        "date": 1773256248412,
        "tool": "cargo",
        "benches": [
          {
            "name": "CreateOTelValueString",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueString",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueInt",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Static",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 139,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 93,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 149,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 28,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "NoAttributes",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithInlineStaticAttributes",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithStaticArray",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes",
            "value": 152,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 135,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 395,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_no_subscriber",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_enabled",
            "value": 395,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 340,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1047,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1602,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 387,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 654,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 411,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1028,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1592,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 20652476,
            "range": "± 708357",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 20600002,
            "range": "± 761343",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 20946329,
            "range": "± 991464",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 21781399,
            "range": "± 1528700",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 24329061,
            "range": "± 1791510",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 23430726,
            "range": "± 886187",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/alt",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/spec",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/alt",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/alt",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/alt",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 6318,
            "range": "± 1526",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 128,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 79,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 81,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 81,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 110,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 113,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 114,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 119,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 165,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 173,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 251,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 253,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 209,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 213,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 336,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 343,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 42,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 109,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 200,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 203,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 331,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 328,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 139,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 126,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 254,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 649,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 72,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 157,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 244,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 457,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 58765,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 119269,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 118,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 284,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 420,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 391,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 314,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 745,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 189,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 267,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 343,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 464,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 198,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 277,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 354,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 475,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 199,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 276,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 354,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 478,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 66,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 227,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 326,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 401,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 493,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 261,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 268,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 398,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 767,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 277,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 289,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 439,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 555,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 162,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 568,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 247,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 810,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 270,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 761,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 363,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 736,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 734,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 215,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 160,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 211,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 261,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 335,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 160,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 410,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 199,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 563,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 243,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 432,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 213,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 608,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 270,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/new_each_time",
            "value": 110,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}