# Test Coverage for Metric Instruments

Tests are located in [mod.rs](mod.rs)

// TODO: Fill this correctly.

## Sync Instruments

| Test Type                  | Counter (Delta) | Counter (Cumulative) | UpDownCounter (Delta) | UpDownCounter (Cumulative) | Gauge (Delta) | Gauge (Cumulative) | Histogram (Delta) | Histogram (Cumulative) |
|----------------------------|-----------------|----------------------|----------------------|----------------------------|---------------|--------------------|-------------------|------------------------|
| Regular aggregation test   | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |
| No-attribute test          | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |
| Overflow test              | [Yes]         | [No]               | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |
| Attr Order Sorted First    | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |
| Attr Order Unsorted First  | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |

## Observable Instruments

| Test Type                  | ObservableCounter (Delta) | ObservableCounter (Cumulative) | ObservableGauge (Delta) | ObservableGauge (Cumulative) | ObservableUpDownCounter (Delta) | ObservableUpDownCounter (Cumulative) |
|----------------------------|---------------------------|-------------------------------|-------------------------|------------------------------|---------------------------------|--------------------------------------|
| Regular aggregation test    | [No]                  | [No]                      | [No]                | [No]                     | [No]                        | [No]                             |
| No-attribute test           | [No]                  | [No]                      | [No]                | [No]                     | [No]                        | [No]                             |
| Attr Order Sorted First    | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |
| Attr Order Unsorted First  | [Yes]         | [Yes]              | [No]              | [No]                    | [No]      | [No]            | [No]           | [No]                |