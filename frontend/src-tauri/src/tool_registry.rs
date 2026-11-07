use serde::Serialize;
use serde_json::{Value, json};
use std::time::Instant;

const MAX_ARGUMENT_BYTES: usize = 64 * 1024;
const MAX_PRICE_SERIES: usize = 10_000;

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub version: u16,
    pub permission_scope: &'static str,
    pub input_schema: Value,
    pub output_schema: Value,
    pub timeout_ms: u64,
    pub availability: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolExecution {
    pub tool_id: &'static str,
    pub result: Value,
    pub result_validated: bool,
    pub independently_verified: bool,
    pub method: &'static str,
    pub elapsed_ms: u128,
}

fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            id: "trading.options.black_scholes.price",
            name: "Black-Scholes price and Greeks",
            description: "Calculate European call/put price and analytic Greeks from explicit inputs.",
            category: "financial_calculation",
            version: 1,
            permission_scope: "local_calculation",
            input_schema: object_schema(
                json!({
                    "kind": {"type": "string", "enum": ["call", "put"]},
                    "spot": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "strike": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "maturity_years": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 100.0},
                    "risk_free_rate": {"type": "number", "minimum": -1.0, "maximum": 1.0},
                    "volatility": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 10.0}
                }),
                &[
                    "kind",
                    "spot",
                    "strike",
                    "maturity_years",
                    "risk_free_rate",
                    "volatility",
                ],
            ),
            output_schema: json!({
                "type": "object",
                "required": ["price", "delta", "gamma", "theta_per_year", "vega_per_volatility_unit", "rho_per_rate_unit"]
            }),
            timeout_ms: 1_000,
            availability: "available",
        },
        ToolDefinition {
            id: "trading.options.implied_volatility.solve",
            name: "Implied volatility solver",
            description: "Solve European Black-Scholes implied volatility by bounded bisection.",
            category: "financial_calculation",
            version: 1,
            permission_scope: "local_calculation",
            input_schema: object_schema(
                json!({
                    "kind": {"type": "string", "enum": ["call", "put"]},
                    "market_price": {"type": "number", "minimum": 0.0, "maximum": 1000000000000.0},
                    "spot": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "strike": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "maturity_years": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 100.0},
                    "risk_free_rate": {"type": "number", "minimum": -1.0, "maximum": 1.0}
                }),
                &[
                    "kind",
                    "market_price",
                    "spot",
                    "strike",
                    "maturity_years",
                    "risk_free_rate",
                ],
            ),
            output_schema: json!({
                "type": "object",
                "required": ["implied_volatility", "unit"]
            }),
            timeout_ms: 1_000,
            availability: "available",
        },
        ToolDefinition {
            id: "trading.risk.position_size",
            name: "Risk-bounded position size",
            description: "Compute whole-unit size from account value, risk percent, entry and stop prices.",
            category: "risk",
            version: 1,
            permission_scope: "local_calculation",
            input_schema: object_schema(
                json!({
                    "account_value": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "risk_percent": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 100.0},
                    "entry_price": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0},
                    "stop_price": {"type": "number", "exclusiveMinimum": 0.0, "maximum": 1000000000000.0}
                }),
                &["account_value", "risk_percent", "entry_price", "stop_price"],
            ),
            output_schema: json!({
                "type": "object",
                "required": ["quantity", "risk_budget", "risk_per_unit", "estimated_loss_at_stop"]
            }),
            timeout_ms: 1_000,
            availability: "available",
        },
        ToolDefinition {
            id: "trading.technical.rsi.calculate",
            name: "RSI",
            description: "Calculate the last available Cutler RSI value from a bounded close-price series.",
            category: "technical_analysis",
            version: 1,
            permission_scope: "local_calculation",
            input_schema: object_schema(
                json!({
                    "closes": {"type": "array", "items": {"type": "number", "exclusiveMinimum": 0.0}, "minItems": 2, "maxItems": MAX_PRICE_SERIES},
                    "period": {"type": "integer", "minimum": 1, "maximum": 500}
                }),
                &["closes", "period"],
            ),
            output_schema: json!({
                "type": "object",
                "required": ["value", "period", "method"]
            }),
            timeout_ms: 1_000,
            availability: "available",
        },
    ]
}

fn is_valid_type(value: &Value, schema_type: &str) -> bool {
    match schema_type {
        "object" => value.is_object(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.as_f64().is_some_and(f64::is_finite),
        _ => false,
    }
}

fn validate_arguments(schema: &Value, arguments: &Value) -> Result<(), String> {
    if !is_valid_type(arguments, "object") {
        return Err("Tool arguments must be a JSON object.".into());
    }
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "Tool schema is invalid.".to_string())?;
    let object = arguments
        .as_object()
        .ok_or_else(|| "Tool arguments must be a JSON object.".to_string())?;
    for required in schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "Tool schema is invalid.".to_string())?
    {
        let key = required
            .as_str()
            .ok_or_else(|| "Tool schema is invalid.".to_string())?;
        if !object.contains_key(key) {
            return Err(format!("Missing required tool argument “{key}”."));
        }
    }
    for (key, value) in object {
        let property_schema = properties
            .get(key)
            .ok_or_else(|| format!("Unknown tool argument “{key}”."))
            .and_then(|schema| {
                schema
                    .as_object()
                    .ok_or_else(|| "Tool schema is invalid.".to_string())
            })?;
        let schema_type = property_schema
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| "Tool schema is invalid.".to_string())?;
        if !is_valid_type(value, schema_type) {
            return Err(format!("Tool argument “{key}” must be {schema_type}."));
        }
        if let Some(choices) = property_schema.get("enum").and_then(Value::as_array)
            && !choices.contains(value)
        {
            return Err(format!("Tool argument “{key}” has an unsupported value."));
        }
        if let Some(number) = value.as_f64() {
            if let Some(minimum) = property_schema.get("minimum").and_then(Value::as_f64)
                && number < minimum
            {
                return Err(format!("Tool argument “{key}” is below its minimum."));
            }
            if let Some(minimum) = property_schema
                .get("exclusiveMinimum")
                .and_then(Value::as_f64)
                && number <= minimum
            {
                return Err(format!(
                    "Tool argument “{key}” must be greater than its minimum."
                ));
            }
            if let Some(maximum) = property_schema.get("maximum").and_then(Value::as_f64)
                && number > maximum
            {
                return Err(format!("Tool argument “{key}” exceeds its maximum."));
            }
        }
        if let Some(array) = value.as_array() {
            let minimum = property_schema
                .get("minItems")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            let maximum = property_schema
                .get("maxItems")
                .and_then(Value::as_u64)
                .unwrap_or(usize::MAX as u64) as usize;
            if array.len() < minimum || array.len() > maximum {
                return Err(format!(
                    "Tool argument “{key}” has an unsupported item count."
                ));
            }
            let item_type = property_schema
                .get("items")
                .and_then(|item| item.get("type"))
                .and_then(Value::as_str)
                .ok_or_else(|| "Tool schema is invalid.".to_string())?;
            if array.iter().any(|item| !is_valid_type(item, item_type)) {
                return Err(format!("Tool argument “{key}” contains an invalid item."));
            }
        }
    }
    Ok(())
}

fn number(arguments: &Value, key: &str) -> Result<f64, String> {
    arguments
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("Tool argument “{key}” must be a finite number."))
}

fn kind(arguments: &Value) -> Result<tredalix_ai::options::OptionKind, String> {
    match arguments.get("kind").and_then(Value::as_str) {
        Some("call") => Ok(tredalix_ai::options::OptionKind::Call),
        Some("put") => Ok(tredalix_ai::options::OptionKind::Put),
        _ => Err("Option kind must be call or put.".into()),
    }
}

pub fn execute(id: &str, arguments: Value) -> Result<ToolExecution, String> {
    let definition = definitions()
        .into_iter()
        .find(|tool| tool.id == id)
        .ok_or_else(|| format!("Unknown tool “{id}”. No action was performed."))?;
    if serde_json::to_vec(&arguments)
        .map_err(|_| "Tool arguments could not be encoded.".to_string())?
        .len()
        > MAX_ARGUMENT_BYTES
    {
        return Err("Tool arguments exceed the 64 KiB limit.".into());
    }
    validate_arguments(&definition.input_schema, &arguments)?;
    let started = Instant::now();
    let (result, verification) = match id {
        "trading.options.black_scholes.price" => {
            let result = tredalix_ai::options::black_scholes(
                kind(&arguments)?,
                number(&arguments, "spot")?,
                number(&arguments, "strike")?,
                number(&arguments, "maturity_years")?,
                number(&arguments, "risk_free_rate")?,
                number(&arguments, "volatility")?,
            )
            .map_err(|error| error.to_string())?;
            (
                serde_json::to_value(result)
                    .map_err(|_| "Option result could not be encoded.".to_string())?,
                "Calculated by the Rust Black-Scholes implementation; European option assumptions apply.",
            )
        }
        "trading.options.implied_volatility.solve" => {
            let volatility = tredalix_ai::options::implied_volatility(
                kind(&arguments)?,
                number(&arguments, "market_price")?,
                number(&arguments, "spot")?,
                number(&arguments, "strike")?,
                number(&arguments, "maturity_years")?,
                number(&arguments, "risk_free_rate")?,
            )
            .map_err(|error| error.to_string())?;
            (
                json!({"implied_volatility": volatility, "unit": "decimal annualized volatility"}),
                "Solved by bounded bisection against the Rust Black-Scholes pricing implementation.",
            )
        }
        "trading.risk.position_size" => {
            let account_value = number(&arguments, "account_value")?;
            let risk_percent = number(&arguments, "risk_percent")?;
            let entry_price = number(&arguments, "entry_price")?;
            let stop_price = number(&arguments, "stop_price")?;
            let risk_per_unit = (entry_price - stop_price).abs();
            if risk_per_unit == 0.0 {
                return Err("Entry and stop prices must differ.".into());
            }
            let risk_budget = account_value * risk_percent / 100.0;
            let quantity = (risk_budget / risk_per_unit).floor();
            (
                json!({
                    "quantity": quantity,
                    "risk_budget": risk_budget,
                    "risk_per_unit": risk_per_unit,
                    "estimated_loss_at_stop": quantity * risk_per_unit
                }),
                "Deterministic whole-unit sizing from supplied account value, risk percentage and stop distance; excludes fees, gaps and liquidity.",
            )
        }
        "trading.technical.rsi.calculate" => {
            let closes = arguments
                .get("closes")
                .and_then(Value::as_array)
                .ok_or_else(|| "Closes must be an array.".to_string())?;
            let values = closes
                .iter()
                .map(|value| {
                    value
                        .as_f64()
                        .filter(|number| number.is_finite())
                        .map(Some)
                        .ok_or_else(|| "Close prices must be finite numbers.".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let period = arguments
                .get("period")
                .and_then(Value::as_u64)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| "RSI period must be a positive integer.".to_string())?;
            let series =
                tredalix_ai::indicators::rsi(&values, period).map_err(|error| error.to_string())?;
            (
                json!({
                    "value": series.last().copied().flatten(),
                    "period": period,
                    "method": "Cutler RSI"
                }),
                "Calculated from the provided close series by the shared Rust indicator engine; no external market data was used.",
            )
        }
        _ => return Err(format!("Tool “{id}” has no registered executor.")),
    };
    if started.elapsed().as_millis() > definition.timeout_ms as u128 {
        return Err("Local calculation exceeded its execution time limit.".into());
    }
    Ok(ToolExecution {
        tool_id: definition.id,
        result,
        result_validated: true,
        independently_verified: false,
        method: verification,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::{definitions, execute};
    use serde_json::json;

    #[test]
    fn tool_ids_are_unique_and_local_math_tools_are_discoverable() {
        let tools = definitions();
        let mut ids = tools.iter().map(|tool| tool.id).collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), tools.len());
        assert!(
            tools
                .iter()
                .any(|tool| tool.id == "trading.options.black_scholes.price")
        );
    }

    #[test]
    fn executes_local_black_scholes_tool_without_claiming_independent_verification() {
        let result = execute(
            "trading.options.black_scholes.price",
            json!({
                "kind": "call",
                "spot": 100.0,
                "strike": 100.0,
                "maturity_years": 1.0,
                "risk_free_rate": 0.05,
                "volatility": 0.2
            }),
        )
        .unwrap();
        assert!(result.result_validated);
        assert!(!result.independently_verified);
        assert!((result.result["price"].as_f64().unwrap() - 10.45058).abs() < 1.0e-4);
    }

    #[test]
    fn rejects_unknown_tools_unexpected_arguments_and_oversized_series() {
        assert!(execute("run_arbitrary_code", json!({})).is_err());
        assert!(
            execute(
                "trading.options.black_scholes.price",
                json!({"kind":"call","spot":100.0,"strike":100.0,"maturity_years":1.0,"risk_free_rate":0.0,"volatility":0.2,"shell":"no"})
            )
            .is_err()
        );
        assert!(
            execute(
                "trading.technical.rsi.calculate",
                json!({"closes":vec![1.0; 10_001],"period":14})
            )
            .is_err()
        );
    }
}
