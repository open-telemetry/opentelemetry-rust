window.BENCHMARK_DATA = {
  "lastUpdate": 1778570108602,
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
      },
      {
        "commit": {
          "author": {
            "name": "Bryant Biggs",
            "username": "bryantbiggs",
            "email": "bryantbiggs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "345cd74a9c88ad1a47435d3d063c12d47235e803",
          "message": "docs: improve with_resource() guidance to preserve SDK defaults (#3418)",
          "timestamp": "2026-03-13T21:12:42Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/345cd74a9c88ad1a47435d3d063c12d47235e803"
        },
        "date": 1773470117597,
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
            "value": 3,
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
            "value": 20,
            "range": "± 1",
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
            "value": 137,
            "range": "± 3",
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
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 146,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 32,
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
            "range": "± 0",
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
            "range": "± 0",
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
            "value": 153,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 136,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 402,
            "range": "± 1",
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
            "value": 390,
            "range": "± 1",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 328,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1072,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1683,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 390,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 645,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 417,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1057,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1588,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 21674805,
            "range": "± 867460",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 20911946,
            "range": "± 999899",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 20943168,
            "range": "± 782078",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 21697763,
            "range": "± 1004330",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 22847235,
            "range": "± 1507347",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 23619341,
            "range": "± 1182198",
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
            "value": 6354,
            "range": "± 3820",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 120,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 63,
            "range": "± 1",
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
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 82,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 79,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 81,
            "range": "± 1",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 125,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 128,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 131,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 161,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 164,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 242,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 245,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 200,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 202,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 329,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 332,
            "range": "± 4",
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
            "range": "± 2",
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
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 202,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 204,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 331,
            "range": "± 2",
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
            "value": 137,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 259,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 648,
            "range": "± 11",
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
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 149,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 222,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 430,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 54699,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 113370,
            "range": "± 12814",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 102,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 291,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 419,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 393,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 320,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 750,
            "range": "± 28",
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
            "value": 201,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 279,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 353,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 478,
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
            "value": 209,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 288,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 361,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 483,
            "range": "± 7",
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
            "value": 211,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 287,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 363,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 485,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 221,
            "range": "± 1",
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
            "value": 404,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 508,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 255,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 275,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 401,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 814,
            "range": "± 3",
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
            "value": 275,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 293,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 438,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 528,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 159,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 546,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 245,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 778,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 275,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 734,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 369,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 724,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 132,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 706,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 217,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 160,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 210,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 264,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 332,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 157,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 416,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 211,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 579,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 243,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 434,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 225,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 613,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 267,
            "range": "± 5",
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
            "value": 107,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Bryant Biggs",
            "username": "bryantbiggs",
            "email": "bryantbiggs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "daaf16ab14d33119858a2a73c86d49ad655fd111",
          "message": "test(appender-tracing): fix flaky experimental_span_attributes tests (#3422)",
          "timestamp": "2026-03-14T21:12:38Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/daaf16ab14d33119858a2a73c86d49ad655fd111"
        },
        "date": 1773556825328,
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
            "value": 20,
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
            "value": 136,
            "range": "± 3",
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
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 146,
            "range": "± 2",
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
            "range": "± 2",
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
            "value": 53,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 54,
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
            "value": 159,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 136,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 400,
            "range": "± 1",
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
            "range": "± 1",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 333,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1059,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1636,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 394,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 657,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 408,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1039,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1554,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 21873224,
            "range": "± 626451",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 21037982,
            "range": "± 1106545",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 21310622,
            "range": "± 1027802",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 22028180,
            "range": "± 1031130",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 22151736,
            "range": "± 623868",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 23039741,
            "range": "± 854475",
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
            "value": 6399,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 120,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 63,
            "range": "± 1",
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
            "value": 80,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 82,
            "range": "± 0",
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
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 81,
            "range": "± 2",
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
            "value": 123,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 128,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 161,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 164,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 241,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 244,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 203,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 204,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 355,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 360,
            "range": "± 1",
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
            "range": "± 0",
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
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 202,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 204,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 334,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 328,
            "range": "± 3",
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
            "value": 136,
            "range": "± 0",
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
            "value": 130,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 259,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 651,
            "range": "± 11",
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
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 150,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 222,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 425,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 57065,
            "range": "± 1347",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 115552,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 106,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 291,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 421,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 393,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 322,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 747,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 37,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 201,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 280,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 353,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 476,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 210,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 287,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 362,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 477,
            "range": "± 2",
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
            "value": 208,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 289,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 361,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 486,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 221,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 325,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 405,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 507,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 256,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 273,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 400,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 820,
            "range": "± 30",
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
            "value": 285,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 294,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 437,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 537,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 161,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 558,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 246,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 773,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 275,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 728,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 360,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 731,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 129,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 706,
            "range": "± 7",
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
            "value": 158,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 210,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 280,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 346,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 160,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 419,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 203,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 590,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 237,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 441,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 215,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 620,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 262,
            "range": "± 5",
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
            "value": 109,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Bryant Biggs",
            "username": "bryantbiggs",
            "email": "bryantbiggs@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "a971b4db532596b7fdf85697f61609ccfdf8f93c",
          "message": "fix(integration-tests): remove per-test file truncation that caused flaky metrics tests (#3424)",
          "timestamp": "2026-03-16T03:12:29Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/a971b4db532596b7fdf85697f61609ccfdf8f93c"
        },
        "date": 1773644026347,
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
            "value": 3,
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
            "value": 20,
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
            "value": 138,
            "range": "± 3",
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
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 146,
            "range": "± 3",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 54,
            "range": "± 1",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 54,
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
            "value": 32,
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
            "value": 149,
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
            "value": 400,
            "range": "± 4",
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
            "value": 415,
            "range": "± 5",
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
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 331,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1048,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1655,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 394,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 642,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 420,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1039,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1578,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 21744363,
            "range": "± 601353",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 20347202,
            "range": "± 932249",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 20538806,
            "range": "± 963578",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 20693235,
            "range": "± 681584",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 21561873,
            "range": "± 667610",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 22722040,
            "range": "± 854811",
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
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 6304,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 120,
            "range": "± 0",
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
            "value": 80,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 79,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 81,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 124,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 125,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 129,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 130,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 162,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 163,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 243,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 244,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 201,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 202,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 347,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 347,
            "range": "± 2",
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
            "value": 43,
            "range": "± 1",
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
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 204,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 204,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 329,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 334,
            "range": "± 4",
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
            "value": 136,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 259,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 650,
            "range": "± 9",
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
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 150,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 220,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 425,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 56356,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 115794,
            "range": "± 628",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 106,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 299,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 419,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 398,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 324,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 748,
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
            "value": 202,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 280,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 353,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 480,
            "range": "± 8",
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
            "value": 209,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 290,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 363,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 484,
            "range": "± 26",
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
            "value": 211,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 289,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 362,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 485,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 221,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 327,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 403,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 507,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 256,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 275,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 401,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 808,
            "range": "± 23",
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
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 291,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 440,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 594,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 160,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 616,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 245,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 804,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 272,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 780,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 359,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 740,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 756,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 218,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 158,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 209,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 264,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 339,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 160,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 422,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 203,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 581,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 235,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 436,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 217,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 630,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 259,
            "range": "± 0",
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
            "value": 106,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Trask Stalnaker",
            "username": "trask",
            "email": "trask.stalnaker@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "4a3aa779bf442c134c5f664092fde913debf3ed4",
          "message": "chore: Migrate to new bare metal runner (Ubuntu 24) (#3425)",
          "timestamp": "2026-03-16T21:51:13Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/4a3aa779bf442c134c5f664092fde913debf3ed4"
        },
        "date": 1773759650815,
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
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueInt",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Static",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 14,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 118,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 81,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 120,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 30,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 137,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 97,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 335,
            "range": "± 13",
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
            "value": 325,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 282,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1052,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1618,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 366,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 522,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 389,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1105,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1576,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 29913289,
            "range": "± 2883832",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 32098902,
            "range": "± 2890742",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 25947495,
            "range": "± 1245947",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26569398,
            "range": "± 3567621",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27770904,
            "range": "± 1047167",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 31570307,
            "range": "± 1815443",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/alt",
            "value": 8,
            "range": "± 1",
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3348,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 65,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 102,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 101,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 105,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 138,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 207,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 210,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 127,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 167,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 285,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 266,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 47,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 156,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 159,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 265,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 270,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 91,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 190,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 592,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 64,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 205,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 401,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 53478,
            "range": "± 4582",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 108181,
            "range": "± 12447",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 254,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 355,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 371,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 281,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 665,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 28,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 174,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 250,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 330,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 338,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 190,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 264,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 348,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 462,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 185,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 261,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 336,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 344,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 45,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 152,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 205,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 347,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 465,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 210,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 214,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 303,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 690,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 229,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 245,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 358,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 434,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 133,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 459,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 198,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 675,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 240,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 610,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 293,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 599,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 81,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 601,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 168,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 138,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 165,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 192,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 261,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 127,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 336,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 179,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 363,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 157,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 349,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 187,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 513,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 174,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "965078315b58ae14725721735f1c8e2bc2d3b445",
          "message": "feat: opentelemetry-otlp: Add provider-agnostic TLS feature for custom crypto backends (#3423)",
          "timestamp": "2026-03-18T20:40:29Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/965078315b58ae14725721735f1c8e2bc2d3b445"
        },
        "date": 1773907555824,
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
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 30,
            "range": "± 2",
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 102,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 62,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 92,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 52,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 49,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 137,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 74,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 330,
            "range": "± 15",
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
            "value": 327,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 282,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1061,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1340,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 365,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 673,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 383,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1063,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1577,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 23841981,
            "range": "± 795778",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 26122273,
            "range": "± 1092958",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 26145194,
            "range": "± 1491142",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26599101,
            "range": "± 988152",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27508452,
            "range": "± 1159250",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 30711494,
            "range": "± 966232",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3375,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 42,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 65,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 66,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 101,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 104,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 103,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 105,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 134,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 206,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 206,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 164,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 137,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 272,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 279,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 55,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 156,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 163,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 265,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 272,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 112,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 69,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 190,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 565,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 64,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 135,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 205,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 400,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 54170,
            "range": "± 5224",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 109098,
            "range": "± 11125",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 74,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 250,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 354,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 359,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 277,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 676,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 22,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 180,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 195,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 335,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 457,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 32,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 182,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 262,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 257,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 449,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 187,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 262,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 259,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 452,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 200,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 274,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 346,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 357,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 20,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 213,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 218,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 304,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 686,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 227,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 247,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 329,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 445,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 155,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 446,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 199,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 685,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 186,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 609,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 289,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 602,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 117,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 593,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 168,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 130,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 198,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 264,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 341,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 168,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 472,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 199,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 359,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 177,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 502,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 220,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 61,
            "range": "± 2",
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
            "value": 107,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Björn Antonsson",
            "username": "bantonsson",
            "email": "bjorn.antonsson@datadoghq.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bb024777b9a197dd8ba9ddf12341a9c2b3a117a4",
          "message": "fix(propagation): clear out unknown trace flags (#3436)",
          "timestamp": "2026-03-27T14:13:45Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/bb024777b9a197dd8ba9ddf12341a9c2b3a117a4"
        },
        "date": 1774679983215,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 105,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 121,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 28,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 28,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 18,
            "range": "± 1",
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
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 97,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 330,
            "range": "± 24",
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
            "value": 318,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 274,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1033,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1644,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 277,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 680,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 378,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1075,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1209,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 29073486,
            "range": "± 2838340",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 31763873,
            "range": "± 2594842",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 26247232,
            "range": "± 1006670",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26600068,
            "range": "± 875421",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27404209,
            "range": "± 952993",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 31021131,
            "range": "± 1846390",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 5,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3332,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 39,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 66,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 100,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 78,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 101,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 138,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 211,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 208,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 164,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 163,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 258,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 266,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 157,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 158,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 208,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 268,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 90,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 188,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 587,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 64,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 139,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 202,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 402,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 53543,
            "range": "± 6044",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 108992,
            "range": "± 12817",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 257,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 362,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 373,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 289,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 693,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 176,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 249,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 251,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 348,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 143,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 202,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 341,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 350,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 181,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 259,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 341,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 454,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 200,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 283,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 360,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 470,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 209,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 212,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 303,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 695,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 233,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 245,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 336,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 437,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 129,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 448,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 147,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 679,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 239,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 614,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 296,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 602,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 613,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 175,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 143,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 179,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 206,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 270,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 134,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 339,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 169,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 489,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 158,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 352,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 183,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 501,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 231,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
            "range": "± 4",
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
            "value": 107,
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
      },
      {
        "commit": {
          "author": {
            "name": "Lalit Kumar Bhasin",
            "username": "lalitb",
            "email": "lalit_fin@yahoo.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b096b70b2ffe9beb65a716cf47d5e5db80a9e930",
          "message": "chore: bump opentelemetry-proto to 0.10.0 (#3443)",
          "timestamp": "2026-03-31T22:22:02Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/b096b70b2ffe9beb65a716cf47d5e5db80a9e930"
        },
        "date": 1775031718959,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 104,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 60,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 122,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 28,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 28,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 47,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 18,
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
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 75,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 333,
            "range": "± 0",
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
            "value": 325,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 274,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 805,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1650,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 364,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 688,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 385,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1069,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1596,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 30069437,
            "range": "± 2755604",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 32066647,
            "range": "± 2385137",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 26105453,
            "range": "± 939616",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26760439,
            "range": "± 1124779",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27791106,
            "range": "± 1041210",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 30542199,
            "range": "± 826046",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 5,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3352,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 53,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 67,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 65,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 67,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 100,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 100,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 102,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 102,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 136,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 212,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 211,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 165,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 278,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 274,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 30,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 158,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 160,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 267,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 271,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 91,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 189,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 568,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 147,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 225,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 440,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 59277,
            "range": "± 6943",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 93026,
            "range": "± 14285",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 257,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 359,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 371,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 286,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 682,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 182,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 254,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 334,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 471,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 38,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 187,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 265,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 341,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 487,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 34,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 181,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 259,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 342,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 476,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 202,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 286,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 371,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 504,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 210,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 215,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 302,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 704,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 9,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 229,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 254,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 356,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 333,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 134,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 450,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 197,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 679,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 246,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 617,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 301,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 609,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 105,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 601,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 164,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 142,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 171,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 197,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 262,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 346,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 180,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 498,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 209,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 277,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 191,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 540,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 229,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
            "range": "± 3",
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
            "value": 108,
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
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67e7dffc3f388c5f39ea1ddf9b4def9c7918679c",
          "message": "chore(deps): bump codecov/codecov-action from 5.5.2 to 6.0.0 (#3447)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-02T00:30:05Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/67e7dffc3f388c5f39ea1ddf9b4def9c7918679c"
        },
        "date": 1775121583058,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 30,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 14,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 103,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 80,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 121,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 30,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 35,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 20,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 18,
            "range": "± 1",
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
            "value": 138,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 96,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 328,
            "range": "± 18",
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
            "value": 320,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 275,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1054,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1654,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 278,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 678,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 383,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1073,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1584,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 29533117,
            "range": "± 3329884",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 31355894,
            "range": "± 2519259",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 32578533,
            "range": "± 2015778",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26743934,
            "range": "± 837769",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27760968,
            "range": "± 736867",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 30978294,
            "range": "± 1176613",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 5,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 2587,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 51,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 50,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 78,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 105,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 103,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 135,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 134,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 205,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 206,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 168,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 164,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 272,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 271,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 30,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 84,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 156,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 158,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 269,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 266,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 115,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 91,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 191,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 540,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 148,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 177,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 437,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 45477,
            "range": "± 7004",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 91801,
            "range": "± 13256",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 101,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 258,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 361,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 365,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 283,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 664,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 179,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 258,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 341,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 482,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 62,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 184,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 267,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 269,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 486,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 36,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 186,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 267,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 350,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 492,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 47,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 155,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 285,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 371,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 510,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 3",
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
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 210,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 214,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 296,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 693,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 227,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 188,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 350,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 448,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 143,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 466,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 208,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 685,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 237,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 622,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 238,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 593,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 106,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 598,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 159,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 157,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 169,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 191,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 258,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 127,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 252,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 172,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 364,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 199,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 349,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 182,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 512,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 220,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
            "range": "± 2",
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
            "value": 108,
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
      },
      {
        "commit": {
          "author": {
            "name": "Marylia Gutierrez",
            "username": "maryliag",
            "email": "maryliag@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "94aa56e5d651696a3ffd388475dc13feee92324c",
          "message": "chore: update readme (#3452)",
          "timestamp": "2026-04-07T18:42:16Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/94aa56e5d651696a3ffd388475dc13feee92324c"
        },
        "date": 1775631173278,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 108,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 83,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 121,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 18,
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
            "value": 137,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 98,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 322,
            "range": "± 7",
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
            "value": 326,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 279,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1041,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1635,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 365,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 688,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1055,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1577,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 31980345,
            "range": "± 2822907",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 34284862,
            "range": "± 2902877",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27972172,
            "range": "± 2524705",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28688444,
            "range": "± 807919",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31293205,
            "range": "± 603130",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 35229978,
            "range": "± 2934586",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 5,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3414,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 68,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 68,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 107,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 101,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 108,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 143,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 138,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 215,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 212,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 169,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 165,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 268,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 268,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 51,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 160,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 158,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 358,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 270,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 110,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 191,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 593,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 64,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 133,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 199,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 398,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 55294,
            "range": "± 6531",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 111650,
            "range": "± 2010",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 250,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 348,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 304,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 283,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 672,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 28,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 168,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 239,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 314,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 419,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 177,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 246,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 322,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 325,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 248,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 317,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 424,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 207,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 269,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 349,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 465,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 226,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 225,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 313,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 733,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 237,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 258,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 354,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 440,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 138,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 461,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 189,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 701,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 239,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 619,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 295,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 606,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 601,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 160,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 142,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 188,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 207,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 202,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 127,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 329,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 174,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 480,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 158,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 349,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 186,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 503,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 174,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
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
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "xofyarg",
            "username": "xofyarg",
            "email": "xofyarg@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "75a0b7e673b4f650159f0cecfd5471e3647aa693",
          "message": "fix(sdk): prevent overdrain race in batch span/log export (#3441)\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-14T05:16:29Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/75a0b7e673b4f650159f0cecfd5471e3647aa693"
        },
        "date": 1776158210827,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 14,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 87,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 60,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 122,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 47,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 54,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 47,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 18,
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
            "value": 107,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 96,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 334,
            "range": "± 23",
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
            "value": 318,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 272,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1033,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1626,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 364,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 675,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1073,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1208,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 24854063,
            "range": "± 1785605",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 26964518,
            "range": "± 2886497",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27475528,
            "range": "± 1118969",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28640851,
            "range": "± 744650",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31011840,
            "range": "± 1304839",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 34237049,
            "range": "± 2263923",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3364,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 99,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 102,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 83,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 137,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 139,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 160,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 213,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 165,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 169,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 268,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 270,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 84,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 155,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 159,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 270,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 274,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 91,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 192,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 580,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 64,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 134,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 208,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 395,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 41952,
            "range": "± 5897",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 108212,
            "range": "± 12910",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 94,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 254,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 358,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 361,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 285,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 670,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 168,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 239,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 312,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 378,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 179,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 188,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 321,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 425,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 36,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 179,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 251,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 247,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 429,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 35,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 147,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 264,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 341,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 448,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 227,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 227,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 308,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 704,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 11,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 230,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 246,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 260,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 456,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 145,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 468,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 195,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 676,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 238,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 611,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 301,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 619,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 596,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 168,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 174,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 199,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 267,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 344,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 185,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 486,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 212,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 362,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 195,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 526,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 235,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
            "range": "± 4",
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
            "value": 107,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8e95e16be60acc64b27f846598267f508b74ce5d",
          "message": "chore: Ignore RUSTSEC-2026-0009 for time crate in deny.toml (#3456)",
          "timestamp": "2026-04-17T20:19:32Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/8e95e16be60acc64b27f846598267f508b74ce5d"
        },
        "date": 1776501126417,
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
            "value": 2,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 105,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 60,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 121,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 52,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 39,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 17,
            "range": "± 8",
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
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 96,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 328,
            "range": "± 23",
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
            "value": 332,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 270,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1052,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1644,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 362,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 680,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1048,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1550,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 27187650,
            "range": "± 731664",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27102802,
            "range": "± 1301145",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27520012,
            "range": "± 1021674",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28898291,
            "range": "± 863162",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31019345,
            "range": "± 2934196",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 34037425,
            "range": "± 2164788",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3369,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 69,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 100,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 106,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 106,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 142,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 162,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 161,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 125,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 167,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 264,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 268,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 39,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 162,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 164,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 272,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 279,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 112,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 88,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 184,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 420,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 134,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 193,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 291,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 50134,
            "range": "± 5062",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 78412,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 252,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 261,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 322,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 278,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 669,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 165,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 191,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 305,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 410,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 175,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 243,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 314,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 424,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 173,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 243,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 315,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 319,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 48,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 193,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 265,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 337,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 441,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 20,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 20,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 205,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 209,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 300,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 696,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 239,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 248,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 324,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 448,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 129,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 366,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 212,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 713,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 253,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 658,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 321,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 607,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 90,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 606,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 163,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 106,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 171,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 202,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 279,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 342,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 170,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 490,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 218,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 357,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 181,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 522,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 240,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
            "range": "± 3",
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
            "value": 108,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Krisztian F",
            "username": "krisztianfekete",
            "email": "103492698+krisztianfekete@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ef8ff2fc0f8f166639fa6c7800b72477a7c6f00f",
          "message": "fix(otlp): surface transport errors at ERROR level (#3463)\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-25T04:46:58Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/ef8ff2fc0f8f166639fa6c7800b72477a7c6f00f"
        },
        "date": 1777100449989,
        "tool": "cargo",
        "benches": [
          {
            "name": "CreateOTelValueString",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueString",
            "value": 2,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 83,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 119,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 48,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 16,
            "range": "± 8",
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
            "value": 134,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 97,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 326,
            "range": "± 7",
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
            "value": 322,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 270,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1027,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1631,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 358,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 667,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1088,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1556,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 25419522,
            "range": "± 3848415",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27090400,
            "range": "± 1210255",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27424085,
            "range": "± 883571",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 29156561,
            "range": "± 1149675",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31564304,
            "range": "± 1663861",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 37487376,
            "range": "± 3455456",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/spec",
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3380,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 67,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 67,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 78,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 102,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 105,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 104,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 136,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 162,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 162,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 169,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 129,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 201,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 266,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 63,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 165,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 167,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 271,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 275,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_concurrent_processor",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 82,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 183,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 591,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 66,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 131,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 188,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 366,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48405,
            "range": "± 5284",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 98401,
            "range": "± 11258",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 243,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 344,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 345,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 276,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 688,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 166,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 233,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 230,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 395,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 175,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 240,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 308,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 400,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 33,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 174,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 241,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 306,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 400,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 34,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 143,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 255,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 325,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 425,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 207,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 216,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 301,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 686,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 235,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 238,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 350,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 431,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 127,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 445,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 185,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 687,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 242,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 614,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 296,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 611,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 112,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 597,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 162,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 138,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 169,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 203,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 264,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 141,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 334,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 168,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 478,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 154,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 352,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 178,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 501,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 222,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
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
            "value": 113,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "329dc2dc135aad921b8b9f29130b1d5613a50b71",
          "message": "chore: Remove SimpleConcurrentLogProcessor (#3471)",
          "timestamp": "2026-04-27T23:25:08Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/329dc2dc135aad921b8b9f29130b1d5613a50b71"
        },
        "date": 1777360573049,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 97,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 60,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 119,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 104,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 97,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 329,
            "range": "± 25",
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
            "value": 322,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 274,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1039,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1630,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 364,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 675,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 284,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1082,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1581,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 27006703,
            "range": "± 1504719",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27308913,
            "range": "± 784490",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27633744,
            "range": "± 898055",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28995994,
            "range": "± 5941875",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31286215,
            "range": "± 2136700",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 37315058,
            "range": "± 3178537",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 20,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3433,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 91,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 40,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 67,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 104,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 106,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 104,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 140,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 140,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 214,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 164,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 162,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 165,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 263,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 262,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 164,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 167,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 273,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 278,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 113,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 108,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 195,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 589,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 71,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 199,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 377,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48261,
            "range": "± 4889",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 98012,
            "range": "± 2536",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 199,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 352,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 356,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 276,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 710,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 41,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 134,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 237,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 309,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 399,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 177,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 243,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 307,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 400,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 32,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 177,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 187,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 309,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 401,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 193,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 257,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 327,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 403,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 211,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 230,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 315,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 715,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 243,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 250,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 360,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 435,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 133,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 454,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 190,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 683,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 246,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 620,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 297,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 605,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 84,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 600,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 122,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 141,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 170,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 194,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 286,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 145,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 334,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 170,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 481,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 207,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 353,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 181,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 511,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 228,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 62,
            "range": "± 5",
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
            "value": 107,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Ophir LOJKINE",
            "username": "lovasoa",
            "email": "contact@ophir.dev"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "146376fde8463a40b807aa68c3d54eab373b3730",
          "message": "fix(opentelemetry-proto): avoid tonic for gen-tonic-messages (#3455)",
          "timestamp": "2026-04-28T18:37:00Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/146376fde8463a40b807aa68c3d54eab373b3730"
        },
        "date": 1777446662183,
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
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 103,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
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
            "value": 82,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 118,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 4",
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
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 58,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 54,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 106,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 336,
            "range": "± 34",
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
            "value": 250,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 278,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1066,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1619,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 374,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 672,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 396,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1054,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1622,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 24908564,
            "range": "± 972539",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 26910383,
            "range": "± 735187",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27683983,
            "range": "± 1290232",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28665912,
            "range": "± 1602328",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31176531,
            "range": "± 2034552",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 33837419,
            "range": "± 1820208",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3386,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 54,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 68,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 68,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 69,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 68,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 101,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 105,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 82,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 138,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 137,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 216,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 213,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 170,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 170,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 273,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 273,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 54,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 165,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 167,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 273,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 276,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 112,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 140,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 594,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 131,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 190,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 367,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48584,
            "range": "± 4535",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 97093,
            "range": "± 11327",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 100,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 262,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 349,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 332,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 267,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 697,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 165,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 176,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 296,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 392,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 173,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 237,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 305,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 397,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 34,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 173,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 247,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 307,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 309,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 47,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 189,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 263,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 327,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 439,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 20,
            "range": "± 3",
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
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 206,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 217,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 316,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 704,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 220,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 238,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 341,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 442,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 130,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 451,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 186,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 692,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 252,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 643,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 310,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 640,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 98,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 621,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 171,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 140,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 178,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 210,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 272,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 141,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 334,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 175,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 468,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 203,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 352,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 179,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 505,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 169,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
            "range": "± 1",
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
            "value": 111,
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
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "34837c839cb8e78e2ac182c047df98c14c3c4c12",
          "message": "docs: Try improve getting started doc (#3299)",
          "timestamp": "2026-04-30T05:08:53Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/34837c839cb8e78e2ac182c047df98c14c3c4c12"
        },
        "date": 1777533472685,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 23,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 103,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 118,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 36,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 48,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 137,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 98,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 54,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 113,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 163,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 135,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 33,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 71,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 337,
            "range": "± 14",
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
            "value": 327,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 271,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1044,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1635,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 354,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 665,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 285,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1041,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1551,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 24946550,
            "range": "± 1077029",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27333361,
            "range": "± 908491",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27269546,
            "range": "± 1209321",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28759261,
            "range": "± 1259614",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31101141,
            "range": "± 1518447",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 34222222,
            "range": "± 1925705",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3363,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 66,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 65,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 78,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 103,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 133,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 160,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 163,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 162,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 263,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 267,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 163,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 165,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 274,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 273,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 184,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 588,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 69,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 134,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 192,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 368,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 47598,
            "range": "± 4421",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 90329,
            "range": "± 11452",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 248,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 346,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 277,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 279,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 693,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 29,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 163,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 220,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 292,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 393,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 33,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 171,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 180,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 305,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 396,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 38,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 133,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 238,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 305,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 401,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 192,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 199,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 320,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 428,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 206,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 209,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 299,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 692,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 236,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 243,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 353,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 443,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 128,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 452,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 186,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 686,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 239,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 621,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 294,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 600,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 112,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 598,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 178,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 169,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 194,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 266,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 132,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 337,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 168,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 453,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 204,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 353,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 180,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 505,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 174,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
            "range": "± 1",
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
            "value": 108,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b1c62dfe49fa5528cad41c25210c41809ef9a136",
          "message": "docs: add continuous benchmark dashboard badge to README (#3483)",
          "timestamp": "2026-04-30T23:48:47Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/b1c62dfe49fa5528cad41c25210c41809ef9a136"
        },
        "date": 1777635330575,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 18,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 79,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 118,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 16,
            "range": "± 2",
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
            "value": 131,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 97,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 113,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 162,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 140,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 31,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 81,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 337,
            "range": "± 0",
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
            "value": 326,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 271,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1047,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1622,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 354,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 678,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 379,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1052,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1578,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 27059580,
            "range": "± 3938236",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27204421,
            "range": "± 753388",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27584119,
            "range": "± 885822",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 29153526,
            "range": "± 1215787",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 31838501,
            "range": "± 1805964",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 34129677,
            "range": "± 1114129",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3391,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 78,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 102,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 103,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 80,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 135,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 210,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 207,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 125,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 125,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 272,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 272,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 40,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 85,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 162,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 167,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 272,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 276,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 184,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 575,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 66,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 136,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 192,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 369,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48071,
            "range": "± 5760",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 96932,
            "range": "± 11529",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 97,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 260,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 352,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 358,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 280,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 683,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 162,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 235,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 292,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 395,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 128,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 180,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 302,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 399,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 37,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 138,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 235,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 313,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 309,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 188,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 257,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 316,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 426,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 27,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 211,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 210,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 301,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 690,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 236,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 246,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 350,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 436,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 130,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 450,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 185,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 688,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 243,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 627,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 303,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 624,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 104,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 612,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 130,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 136,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 133,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 202,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 265,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 125,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 337,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 172,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 487,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 199,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 354,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 182,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 514,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 169,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
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
            "value": 108,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "cef3317759a03698420bbbc0939afcad8bcd453c",
          "message": "chore(deps): bump actions/upload-artifact from 7.0.0 to 7.0.1 (#3488)\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-01T23:26:34Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/cef3317759a03698420bbbc0939afcad8bcd453c"
        },
        "date": 1777753103583,
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
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueInt",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Static",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 19,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 12,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 105,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 80,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 118,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 16,
            "range": "± 1",
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
            "value": 161,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 100,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 53,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 53,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 115,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 171,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 138,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 72,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 83,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 355,
            "range": "± 29",
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
            "value": 330,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 269,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1043,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1610,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 356,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 668,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 375,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1040,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1551,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 25174986,
            "range": "± 1184406",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 27267160,
            "range": "± 1336740",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 27902977,
            "range": "± 1647298",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 28944119,
            "range": "± 2737475",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 30981780,
            "range": "± 1224977",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 34138391,
            "range": "± 1457502",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 20,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3406,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 66,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 66,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 66,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 66,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 99,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 100,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 79,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 133,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 134,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 208,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 212,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 165,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 165,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 264,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 208,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 40,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 82,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 162,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 126,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 272,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 302,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 111,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 184,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 596,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 51,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 132,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 147,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 381,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48315,
            "range": "± 4924",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 96889,
            "range": "± 6623",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 98,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 249,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 343,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 319,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 278,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 685,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 22,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 163,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 227,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 291,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 392,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 171,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 181,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 302,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 401,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 32,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 172,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 245,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 304,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 403,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 188,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 252,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 317,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 324,
            "range": "± 49",
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
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 205,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 159,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 228,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 691,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 236,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 243,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 348,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 444,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 133,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 467,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 204,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 681,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 251,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 621,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 292,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 607,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 79,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 607,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 160,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 129,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 164,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 193,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 267,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 338,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 167,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 475,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 199,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 357,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 179,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 506,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 206,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
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
            "value": 108,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c9923793652d03c20ccfaf39cfa9273243152843",
          "message": "docs: Improve getting-started example discoverability (#3479)",
          "timestamp": "2026-05-04T13:34:54Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/c9923793652d03c20ccfaf39cfa9273243152843"
        },
        "date": 1777964897647,
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
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 80,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 61,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 119,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
            "range": "± 1",
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
            "value": 135,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 96,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 51,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 53,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 113,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 167,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 136,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 32,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 72,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 78,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 336,
            "range": "± 22",
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
            "value": 321,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 272,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1032,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1619,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 363,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 671,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 800,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1554,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 30069155,
            "range": "± 2369163",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 31588112,
            "range": "± 2244827",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 25758557,
            "range": "± 1532824",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 25952671,
            "range": "± 1025538",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27184376,
            "range": "± 675168",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 31453418,
            "range": "± 1555732",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 4,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3426,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 90,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 65,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 69,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 69,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 65,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 68,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 103,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 106,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 105,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 108,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 140,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 142,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 206,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 208,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 163,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 164,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 257,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 259,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 63,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 162,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 165,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 272,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 357,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 116,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 112,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 87,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 183,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 575,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 133,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 191,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 369,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48364,
            "range": "± 5162",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 97795,
            "range": "± 10213",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 96,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 244,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 343,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 357,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 287,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 529,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 164,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 230,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 296,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 299,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 173,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 237,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 301,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 402,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 184,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 243,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 305,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 307,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 143,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 252,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 317,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 425,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 29,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 206,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 209,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 294,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 679,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 220,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 246,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 340,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 435,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 127,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 442,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 185,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 692,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 248,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 625,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 299,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 619,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 120,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 604,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 171,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 131,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 165,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 189,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 270,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 135,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 334,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 168,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 474,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 205,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 349,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 138,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 504,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 174,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 63,
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
            "value": 107,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Rory",
            "username": "rornic",
            "email": "rorynickolls@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "04b6a6fd06f7c381cdfe15cf461a39308d87a182",
          "message": "fix: prevent logging of header values in tonic debug logs (#3465)",
          "timestamp": "2026-05-05T20:23:10Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/04b6a6fd06f7c381cdfe15cf461a39308d87a182"
        },
        "date": 1778051929560,
        "tool": "cargo",
        "benches": [
          {
            "name": "CreateOTelValueString",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelAnyValueString",
            "value": 3,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 102,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 117,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 35,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
            "range": "± 2",
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
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 52,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 113,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 169,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 137,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 72,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 78,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 335,
            "range": "± 4",
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
            "value": 249,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 272,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1049,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1621,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 358,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 671,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 375,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1042,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1551,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 23245494,
            "range": "± 836785",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 26190702,
            "range": "± 3526601",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 26076761,
            "range": "± 1326235",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26431349,
            "range": "± 902014",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27388810,
            "range": "± 835476",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 32056145,
            "range": "± 1090308",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Unbound_Delta",
            "value": 103,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_With_View_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_AtOverflow_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Unbound_Delta",
            "value": 118,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Bound_Delta",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Bound_AtOverflow_Delta",
            "value": 18,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/2",
            "value": 136853,
            "range": "± 99365",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/4",
            "value": 399104,
            "range": "± 70104",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/8",
            "value": 606451,
            "range": "± 28886",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 20,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3373,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 92,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 56,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 66,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 67,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 67,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 67,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 103,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 102,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 104,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 80,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 137,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 141,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 205,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 158,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 161,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 168,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 263,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 266,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 52,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 55,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 32,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 85,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 166,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 170,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 277,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 215,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 114,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 109,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 89,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 142,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 591,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 70,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 142,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 206,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 384,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 51656,
            "range": "± 5600",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 80924,
            "range": "± 12354",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 101,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 251,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 351,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 364,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 280,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 686,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 171,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 236,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 308,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 429,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 33,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 174,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 246,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 317,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 431,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 174,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 248,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 314,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 441,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 44,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 193,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 269,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 341,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 450,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 37,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 37,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 213,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 221,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 311,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 730,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 236,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 243,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 344,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 431,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 119,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 440,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 174,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 668,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 230,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 600,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 276,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 566,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 90,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 572,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 144,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 131,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 154,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 185,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 246,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 108,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 308,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 147,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 344,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 179,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 323,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 156,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 479,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 201,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 94,
            "range": "± 9",
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
            "value": 139,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "5a07ce159580dc91d1bb7b9b56b0082d1f74a414",
          "message": "ci: close stale pull requests (#3499)",
          "timestamp": "2026-05-07T01:03:54Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/5a07ce159580dc91d1bb7b9b56b0082d1f74a414"
        },
        "date": 1778139232676,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 63,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 116,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 21,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
            "range": "± 1",
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
            "value": 136,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 98,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 51,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 40,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 112,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 163,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 134,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 70,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 337,
            "range": "± 29",
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
            "value": 322,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 283,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1038,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1632,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 361,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 681,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 377,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1065,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1550,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 1 concurrent task",
            "value": 29115916,
            "range": "± 2803957",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 2 concurrent task",
            "value": 31202042,
            "range": "± 2180805",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 4 concurrent task",
            "value": 25827774,
            "range": "± 887848",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 8 concurrent task",
            "value": 26395079,
            "range": "± 801990",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 16 concurrent task",
            "value": 27552734,
            "range": "± 1214190",
            "unit": "ns/iter"
          },
          {
            "name": "BatchSpanProcessor/with 32 concurrent task",
            "value": 30913250,
            "range": "± 1356306",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Unbound_Delta",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_With_View_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_AtOverflow_Delta",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Unbound_Delta",
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Bound_Delta",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Histogram_Bound_AtOverflow_Delta",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/2",
            "value": 230396,
            "range": "± 45157",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/4",
            "value": 262188,
            "range": "± 18567",
            "unit": "ns/iter"
          },
          {
            "name": "BoundInstruments/Counter_Bound_Multithread/8",
            "value": 524037,
            "range": "± 46235",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/in-cx/alt",
            "value": 7,
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
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/in-cx/spec",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/in-cx/spec",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-cx/alt",
            "value": 7,
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
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-cx/spec",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/has_active_span/no-sdk/alt",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/alt",
            "value": 6,
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
            "value": 4,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_sampled/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/is_recording/no-sdk/spec",
            "value": 5,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Logger_Creation",
            "value": 22,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "LoggerProvider_Creation",
            "value": 3347,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "Logging_Comparable_To_Appender",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/no-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log/with-context",
            "value": 53,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/no-context",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-int/with-context",
            "value": 68,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/no-context",
            "value": 67,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-double/with-context",
            "value": 70,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/no-context",
            "value": 68,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-string/with-context",
            "value": 70,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/no-context",
            "value": 67,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bool/with-context",
            "value": 70,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/no-context",
            "value": 102,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-bytes/with-context",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/no-context",
            "value": 81,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-a-lot-of-bytes/with-context",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/no-context",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-vec-any-value/with-context",
            "value": 138,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/no-context",
            "value": 213,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-vec-any-value/with-context",
            "value": 162,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/no-context",
            "value": 163,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-map-any-value/with-context",
            "value": 164,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/no-context",
            "value": 257,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "simple-log-with-inner-map-any-value/with-context",
            "value": 255,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/no-context",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "long-log/with-context",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/no-context",
            "value": 30,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log/with-context",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/no-context",
            "value": 84,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-4-attributes/with-context",
            "value": 89,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/no-context",
            "value": 168,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-9-attributes/with-context",
            "value": 168,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/no-context",
            "value": 278,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "full-log-with-attributes/with-context",
            "value": 278,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "exporter_disabled_simple_processor",
            "value": 18,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithFuture",
            "value": 113,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "LogExporterWithoutFuture",
            "value": 108,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "log_noop_processor",
            "value": 89,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "log_cloning_processor",
            "value": 194,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "log_clone_and_send_to_channel_processor",
            "value": 591,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddNoAttrs",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneAttr",
            "value": 67,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddThreeAttr",
            "value": 136,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddFiveAttr",
            "value": 189,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddTenAttr",
            "value": 362,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddOneTillMaxAttr",
            "value": 48307,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddMaxAttr",
            "value": 75467,
            "range": "± 8804",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddInvalidAttr",
            "value": 97,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseAttrs",
            "value": 252,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseInvalid",
            "value": 356,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/AddSingleUseFiltered",
            "value": 351,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectOneAttr",
            "value": 288,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Counter/CollectTenAttrs",
            "value": 704,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs10bounds",
            "value": 29,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs10bounds",
            "value": 166,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs10bounds",
            "value": 179,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs10bounds",
            "value": 300,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs10bounds",
            "value": 399,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs49bounds",
            "value": 34,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs49bounds",
            "value": 175,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs49bounds",
            "value": 243,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs49bounds",
            "value": 309,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs49bounds",
            "value": 407,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs50bounds",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs50bounds",
            "value": 174,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs50bounds",
            "value": 244,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs50bounds",
            "value": 310,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs50bounds",
            "value": 407,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record0Attrs1000bounds",
            "value": 47,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record3Attrs1000bounds",
            "value": 194,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record5Attrs1000bounds",
            "value": 263,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record7Attrs1000bounds",
            "value": 332,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/Record10Attrs1000bounds",
            "value": 434,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectOne",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectFive",
            "value": 26,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTen",
            "value": 36,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram/CollectTwentyFive",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted",
            "value": 209,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Unsorted",
            "value": 212,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Add_Sorted_With_Non_Static_Values",
            "value": 305,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Counter_Overflow",
            "value": 696,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "ThreadLocal_Random_Generator_5",
            "value": 10,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Gauge_Add",
            "value": 229,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record",
            "value": 253,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Histogram_Record_With_Non_Static_Values",
            "value": 341,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/always-sample",
            "value": 451,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple/never-sample",
            "value": 125,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/always-sample",
            "value": 346,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder/never-sample",
            "value": 171,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/always-sample",
            "value": 545,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span/never-sample",
            "value": 178,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/always-sample",
            "value": 628,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-tracer-in-span-with-builder/never-sample",
            "value": 276,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/always-sample",
            "value": 599,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-simple-context-activation/never-sample",
            "value": 90,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/always-sample",
            "value": 615,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "span-creation-span-builder-context-activation/never-sample",
            "value": 147,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/simplest",
            "value": 122,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/1",
            "value": 151,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "span_builder/with_attributes/4",
            "value": 180,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/always-sample",
            "value": 274,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span/never-sample",
            "value": 89,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/always-sample",
            "value": 337,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-4-attrs/never-sample",
            "value": 144,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/always-sample",
            "value": 477,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-8-attrs/never-sample",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/always-sample",
            "value": 352,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types/never-sample",
            "value": 155,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/always-sample",
            "value": 506,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "start-end-span-all-attr-types-2x/never-sample",
            "value": 200,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name/new_each_time",
            "value": 94,
            "range": "± 5",
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
            "value": 141,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Tracer_With_Name_And_Scope_Attrs/reuse_existing",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f744509915e6e3b4fc2b551fd0c83f6a96e1fc71",
          "message": "docs: update README status table and remove deprecated crates (#3502)",
          "timestamp": "2026-05-07T18:12:32Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/f744509915e6e3b4fc2b551fd0c83f6a96e1fc71"
        },
        "date": 1778229779132,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 23,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 103,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 60,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 82,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 116,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 48,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 28,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 52,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
            "range": "± 2",
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
            "value": 137,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 74,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 51,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 51,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 53,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 55,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 112,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 162,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 134,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 30,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 71,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 76,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 336,
            "range": "± 30",
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
            "value": 326,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 15,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 24,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 10,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 284,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1044,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1637,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 363,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 681,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 384,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1099,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1609,
            "range": "± 189",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ec289cb3c6f8260951699c51df968560943c1451",
          "message": "chore: Prepare for release v0.32.0 (#3508)",
          "timestamp": "2026-05-08T23:28:58Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/ec289cb3c6f8260951699c51df968560943c1451"
        },
        "date": 1778356777164,
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
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 80,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 35,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 53,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 47,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 21,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
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
            "value": 136,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 96,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 41,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 55,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 87,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 164,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 135,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 69,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 79,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 328,
            "range": "± 27",
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
            "value": 328,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 274,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1053,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1308,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 356,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 679,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 379,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1072,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1571,
            "range": "± 136",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cijo.thomas@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dfdc478462ba345d1eb5f158d061bc8b20d33f04",
          "message": "fix: break circular dev-dep between appender-tracing and stdout (#3509)",
          "timestamp": "2026-05-11T14:42:40Z",
          "url": "https://github.com/open-telemetry/opentelemetry-rust/commit/dfdc478462ba345d1eb5f158d061bc8b20d33f04"
        },
        "date": 1778570107583,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelValueInt",
            "value": 1,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Owned",
            "value": 17,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKey_Arc",
            "value": 31,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOTelKeyValue",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValue",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArray",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithMixedValueTypes",
            "value": 13,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "CreateOtelKeyValueArrayWithNonStaticValues",
            "value": 105,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateTupleKeyValueArray",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key_value",
            "value": 30,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_static_key",
            "value": 59,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic",
            "value": 83,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "set_baggage_dynamic_with_metadata",
            "value": 119,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/empty_cx",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/empty_cx",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/empty_cx",
            "value": 35,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/single_value_cx",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/single_value_cx",
            "value": 40,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/single_value_cx",
            "value": 46,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/single_cx/span_cx",
            "value": 27,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/nested_cx/span_cx",
            "value": 53,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context_attach/out_of_order_cx_drop/span_cx",
            "value": 46,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/enter_telemetry_suppressed_scope",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/normal_attach",
            "value": 28,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_false",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "telemetry_suppression/is_current_telemetry_suppressed_true",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithAttributes",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_WithBody",
            "value": 3,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateLogRecord_NoopLogger_Full",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "EventEnabled_NoopLogger",
            "value": 0,
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
            "value": 17,
            "range": "± 1",
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
            "value": 103,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "AddWithDynamicAttributes_WithStringAllocation",
            "value": 118,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer",
            "value": 51,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_4Attributes",
            "value": 51,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddEvent",
            "value": 52,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_AddLink",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_SetActive",
            "value": 112,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_WithActiveParent",
            "value": 164,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "CreateSpan_NoopTracer_InSpan",
            "value": 134,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_Creation",
            "value": 31,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithAttributes",
            "value": 69,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "SpanBuilder_WithLinks",
            "value": 82,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "otel_2_attributes",
            "value": 326,
            "range": "± 27",
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
            "value": 329,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "ot_layer_disabled",
            "value": 16,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_enabled",
            "value": 23,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "noop_layer_disabled",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_no_span",
            "value": 275,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_span_2_attr",
            "value": 1058,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "log_1_attr_in_nested_spans_2plus2_attr",
            "value": 1614,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "span_4_attributes",
            "value": 356,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "span_8_attributes",
            "value": 668,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_1_levels",
            "value": 371,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_2_levels",
            "value": 1055,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "nested_spans_3_levels",
            "value": 1541,
            "range": "± 162",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}