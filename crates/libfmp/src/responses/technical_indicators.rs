//! Response rows returned by technical-indicator endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{ApiDateTime, Price, Volume};

macro_rules! technical_indicator_row {
    ($name:ident, $metric:ident, $metric_type:ty) => {
        #[doc = concat!("One ", stringify!($metric), " technical-indicator row.")]
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub date: ApiDateTime,
            pub open: Price,
            pub high: Price,
            pub low: Price,
            pub close: Price,
            #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
            pub volume: Volume,
            pub $metric: $metric_type,
        }
    };
}

technical_indicator_row!(SimpleMovingAverageBar, sma, Price);
technical_indicator_row!(ExponentialMovingAverageBar, ema, Price);
technical_indicator_row!(WeightedMovingAverageBar, wma, Price);
technical_indicator_row!(DoubleExponentialMovingAverageBar, dema, Price);
technical_indicator_row!(TripleExponentialMovingAverageBar, tema, Price);
technical_indicator_row!(RelativeStrengthIndexBar, rsi, f64);
technical_indicator_row!(StandardDeviationBar, standard_deviation, Price);
technical_indicator_row!(WilliamsBar, williams, f64);
technical_indicator_row!(AverageDirectionalIndexBar, adx, f64);
