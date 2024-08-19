# Test Coverage for Metric Instruments

Tests are located in [mod.rs](mod.rs)

// TODO: Fill this correctly.

:heavy_check_mark: 

## Sync Instruments

| Test Type                  | Counter (Delta) | Counter (Cumulative) | UpDownCounter (Delta) | UpDownCounter (Cumulative) | Gauge (Delta) | Gauge (Cumulative) | Histogram (Delta) | Histogram (Cumulative) |
|----------------------------|-----------------|----------------------|----------------------|----------------------------|---------------|--------------------|-------------------|------------------------|
| Regular aggregation test   | ✅        | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |
| No-attribute test          | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |
| Overflow test              | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |
| Attr Order Sorted First    | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |
| Attr Order Unsorted First  | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |

## Observable Instruments

| Test Type                  | ObservableCounter (Delta) | ObservableCounter (Cumulative) | ObservableGauge (Delta) | ObservableGauge (Cumulative) | ObservableUpDownCounter (Delta) | ObservableUpDownCounter (Cumulative) |
|----------------------------|---------------------------|-------------------------------|-------------------------|------------------------------|---------------------------------|--------------------------------------|
| Regular aggregation test    | ❌                  | ❌                      | ❌                | ❌                     | ❌                        | ❌                             |
| No-attribute test           | ❌                  | ❌                      | ❌                | ❌                     | ❌                        | ❌                             |
| Attr Order Sorted First    | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |
| Attr Order Unsorted First  | ✅         | ✅              | ❌              | ❌                    | ❌      | ❌            | ❌           | ❌                |