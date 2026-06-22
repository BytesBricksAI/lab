# sp_dataframe_query

SimPlant Lab dataframe query adapter: implements [`DataframeQueryPort`] from `sp_ml_dataloop` by reading `.rrd` recordings through [`re_dataframe`].

This crate is the anti-corruption boundary between domain query types (`TimeWindow`, `TagId`, `Measurement`) and the Rerun chunk store / dataframe APIs.

## Usage

```rust
use sp_dataframe_query::RrdDataframeQuery;
use sp_ml_dataloop::DataframeQueryPort;

let query = RrdDataframeQuery::open("recording.rrd")?;
let result = query.query(&window, &[tag_a, tag_b])?;
```

[`DataframeQueryPort`]: https://docs.rs/sp_ml_dataloop/latest/sp_ml_dataloop/trait.DataframeQueryPort.html
[`re_dataframe`]: https://docs.rs/re_dataframe/latest/re_dataframe/
