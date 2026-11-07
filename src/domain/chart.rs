use serde::{Deserialize, Serialize};

use super::CurrencyCode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorPoint {
    pub timestamp_utc: i64,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorSeries {
    pub name: String,
    pub calculation_version: String,
    pub parameters: Vec<(String, String)>,
    pub warmup_bars: usize,
    pub points: Vec<IndicatorPoint>,
}

impl IndicatorSeries {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.calculation_version.trim().is_empty() {
            return Err("Indicator name and calculation version are required".into());
        }
        let mut previous = None;
        for point in &self.points {
            if previous.is_some_and(|timestamp| point.timestamp_utc <= timestamp) {
                return Err("Indicator timestamps must be strictly increasing".into());
            }
            if point.value.is_some_and(|value| !value.is_finite()) {
                return Err("Indicator values must be finite or explicitly missing".into());
            }
            previous = Some(point.timestamp_utc);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayOwner {
    User,
    Indicator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OverlayGeometry {
    HorizontalLine {
        price: f64,
    },
    TrendLine {
        start_timestamp_utc: i64,
        start_price: f64,
        end_timestamp_utc: i64,
        end_price: f64,
    },
    Rectangle {
        start_timestamp_utc: i64,
        end_timestamp_utc: i64,
        low: f64,
        high: f64,
    },
    Marker {
        timestamp_utc: i64,
        price: f64,
        label: String,
    },
    Annotation {
        timestamp_utc: i64,
        price: f64,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartOverlay {
    pub id: String,
    pub owner: OverlayOwner,
    pub currency: CurrencyCode,
    pub geometry: OverlayGeometry,
}

impl ChartOverlay {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() || self.id.len() > 128 {
            return Err("Overlay ID must be 1-128 characters".into());
        }
        let finite_positive = |price: f64| price.is_finite() && price > 0.0;
        match &self.geometry {
            OverlayGeometry::HorizontalLine { price } => {
                if !finite_positive(*price) {
                    return Err("Overlay price must be finite and positive".into());
                }
            }
            OverlayGeometry::TrendLine {
                start_timestamp_utc,
                start_price,
                end_timestamp_utc,
                end_price,
            } => {
                if start_timestamp_utc >= end_timestamp_utc
                    || !finite_positive(*start_price)
                    || !finite_positive(*end_price)
                {
                    return Err("Trend line coordinates are invalid".into());
                }
            }
            OverlayGeometry::Rectangle {
                start_timestamp_utc,
                end_timestamp_utc,
                low,
                high,
            } => {
                if start_timestamp_utc >= end_timestamp_utc
                    || !finite_positive(*low)
                    || !finite_positive(*high)
                    || low > high
                {
                    return Err("Rectangle coordinates are invalid".into());
                }
            }
            OverlayGeometry::Marker { price, label, .. } => {
                if !finite_positive(*price) || label.trim().is_empty() {
                    return Err("Marker price and label are required".into());
                }
            }
            OverlayGeometry::Annotation { price, text, .. } => {
                if !finite_positive(*price) || text.trim().is_empty() {
                    return Err("Annotation price and text are required".into());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ChartOverlay, IndicatorPoint, IndicatorSeries, OverlayGeometry, OverlayOwner};
    use crate::domain::CurrencyCode;

    #[test]
    fn indicator_timestamps_are_explicit_and_must_be_ordered() {
        let series = IndicatorSeries {
            name: "SMA".into(),
            calculation_version: "1".into(),
            parameters: vec![("period".into(), "2".into())],
            warmup_bars: 1,
            points: vec![
                IndicatorPoint {
                    timestamp_utc: 1,
                    value: None,
                },
                IndicatorPoint {
                    timestamp_utc: 2,
                    value: Some(10.0),
                },
            ],
        };
        series.validate().unwrap();
        let mut invalid = series;
        invalid.points[1].timestamp_utc = 1;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn overlay_coordinates_validate_and_do_not_embed_forecasts() {
        let overlay = ChartOverlay {
            id: "user-level-1".into(),
            owner: OverlayOwner::User,
            currency: CurrencyCode::new("INR").unwrap(),
            geometry: OverlayGeometry::HorizontalLine { price: 100.0 },
        };
        overlay.validate().unwrap();
        let encoded = serde_json::to_string(&overlay).unwrap();
        assert!(encoded.contains("horizontal_line"));

        let invalid = ChartOverlay {
            geometry: OverlayGeometry::Rectangle {
                start_timestamp_utc: 10,
                end_timestamp_utc: 5,
                low: 90.0,
                high: 100.0,
            },
            ..overlay
        };
        assert!(invalid.validate().is_err());
    }
}
