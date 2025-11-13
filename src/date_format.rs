use chrono::{DateTime, Datelike, Utc};
use std::collections::HashMap;
use tera::{Result as TeraResult, Value};

/// Get month name in the specified language
pub fn get_month_name(month: u32, lang: &str) -> &'static str {
    if lang == "en" {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    } else {
        match month {
            1 => "enero",
            2 => "febrero",
            3 => "marzo",
            4 => "abril",
            5 => "mayo",
            6 => "junio",
            7 => "julio",
            8 => "agosto",
            9 => "septiembre",
            10 => "octubre",
            11 => "noviembre",
            12 => "diciembre",
            _ => "Unknown",
        }
    }
}

/// Format a date string according to the specified language
/// This function is used as a Tera template filter
pub fn date_format(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let date_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("Expected a string for date"))?;
    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| tera::Error::msg(format!("Invalid date format: {}", e)))?
        .with_timezone(&Utc);

    // Get language from args, default to Spanish
    let lang = args
        .get("lang")
        .and_then(|v| v.as_str())
        .unwrap_or("es");

    let month_name = get_month_name(date.month(), lang);

    let formatted = if lang == "en" {
        format!("{} {}, {}", month_name, date.day(), date.year())
    } else {
        format!("{:02} de {} de {}", date.day(), month_name, date.year())
    };
    Ok(Value::String(formatted))
}

