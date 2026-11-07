use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::model::Candle;
use super::timestamp::parse_timestamp;
use super::validation::validate_candle;

pub fn load_csv(path: impl AsRef<Path>) -> Result<Vec<Candle>, String> {
    let file = File::open(path.as_ref())
        .map_err(|error| format!("Cannot open CSV '{}': {error}", path.as_ref().display()))?;
    parse_reader(BufReader::new(file))
}

pub fn parse_csv(contents: &str) -> Result<Vec<Candle>, String> {
    parse_reader(BufReader::new(contents.as_bytes()))
}

fn parse_reader(reader: impl BufRead) -> Result<Vec<Candle>, String> {
    let mut lines = reader.lines().enumerate();
    let (_, first_line) = lines.next().ok_or("CSV empty hai")?;
    let header = first_line.map_err(|error| format!("Cannot read CSV header: {error}"))?;
    let header = header.trim_start_matches('\u{feff}');
    let headers = parse_record(header).map_err(|error| format!("Invalid CSV header: {error}"))?;
    let mut indexes = Vec::with_capacity(headers.len());
    for header in headers {
        let normalized = header.trim().to_ascii_lowercase();
        if indexes.iter().any(|(name, _)| name == &normalized) {
            return Err(format!("Duplicate CSV header: {normalized}"));
        }
        indexes.push((normalized, indexes.len()));
    }

    let column = |name: &str| {
        indexes
            .iter()
            .find(|(header, _)| header == name)
            .map(|(_, index)| *index)
            .ok_or_else(|| format!("Required CSV column missing: {name}"))
    };
    let timestamp_index = column("timestamp")?;
    let open_index = column("open")?;
    let high_index = column("high")?;
    let low_index = column("low")?;
    let close_index = column("close")?;
    let volume_index = indexes
        .iter()
        .find(|(header, _)| header == "volume")
        .map(|(_, index)| *index);

    let mut candles = Vec::new();
    let mut previous_timestamp = None;
    for (line_index, line) in lines {
        let line_number = line_index + 1;
        let line = line.map_err(|error| format!("Row {line_number}: cannot read: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_record(&line).map_err(|error| format!("Row {line_number}: {error}"))?;
        if fields.len() != indexes.len() {
            return Err(format!(
                "Row {line_number}: expected {} fields, found {}",
                indexes.len(),
                fields.len()
            ));
        }

        let number = |index: usize, field: &str| -> Result<f64, String> {
            fields[index]
                .trim()
                .parse()
                .map_err(|_| format!("Row {line_number}: invalid {field} value"))
        };
        let timestamp = parse_timestamp(&fields[timestamp_index])
            .map_err(|error| format!("Row {line_number}: {error}"))?;
        if let Some(previous) = previous_timestamp {
            if timestamp == previous {
                return Err(format!("Row {line_number}: duplicate timestamp"));
            }
            if timestamp < previous {
                return Err(format!("Row {line_number}: timestamps are not ascending"));
            }
        }
        let candle = Candle {
            timestamp,
            open: number(open_index, "open")?,
            high: number(high_index, "high")?,
            low: number(low_index, "low")?,
            close: number(close_index, "close")?,
            volume: match volume_index {
                Some(index) if !fields[index].trim().is_empty() => Some(number(index, "volume")?),
                _ => None,
            },
        };
        validate_candle(&candle).map_err(|error| format!("Row {line_number}: {error}"))?;
        previous_timestamp = Some(timestamp);
        candles.push(candle);
    }

    if candles.is_empty() {
        return Err("CSV has a header but no data rows".to_owned());
    }
    Ok(candles)
}

fn parse_record(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut quoted = false;
    let mut quote_closed = false;

    while let Some(character) = chars.next() {
        if quoted {
            match character {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next();
                    field.push('"');
                }
                '"' => {
                    quoted = false;
                    quote_closed = true;
                }
                _ => field.push(character),
            }
        } else if quote_closed {
            match character {
                ',' => {
                    fields.push(std::mem::take(&mut field));
                    quote_closed = false;
                }
                value if value.is_whitespace() => {}
                _ => return Err("unexpected character after quoted field".to_owned()),
            }
        } else {
            match character {
                ',' => fields.push(std::mem::take(&mut field)),
                '"' if field.trim().is_empty() => {
                    field.clear();
                    quoted = true;
                }
                '"' => return Err("quote inside an unquoted field".to_owned()),
                _ => field.push(character),
            }
        }
    }
    if quoted {
        return Err("unclosed quoted field".to_owned());
    }
    fields.push(field);
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::parse_record;

    #[test]
    fn parses_quoted_commas_and_escaped_quotes() {
        assert_eq!(
            parse_record(r#"one,"two, ""quoted""",three"#).unwrap(),
            ["one", "two, \"quoted\"", "three"]
        );
    }

    #[test]
    fn rejects_malformed_quotes() {
        assert!(parse_record(r#"one,"unfinished"#).is_err());
        assert!(parse_record(r#"one,"closed"x"#).is_err());
        assert!(parse_record(r#"un"escaped"#).is_err());
    }
}
