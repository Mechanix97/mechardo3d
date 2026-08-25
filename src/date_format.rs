use chrono::{DateTime, Datelike, NaiveDate, Utc};
use std::collections::HashMap;
use tera::{Result as TeraResult, Value};

use crate::language::Language;

/// Month name in the given language.
pub fn get_month_name(month: u32, lang: Language) -> &'static str {
    const ENGLISH: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    const SPANISH: [&str; 12] = [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ];

    let index = month.checked_sub(1).unwrap_or(12) as usize;
    let names = match lang {
        Language::English => &ENGLISH,
        Language::Spanish => &SPANISH,
    };
    names.get(index).copied().unwrap_or("Unknown")
}

/// Tera filter: `{{ post.date | date_format(lang=lang) }}`.
pub fn date_format(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let date_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("Expected a string for date"))?;
    let date = parse_date(date_str)
        .ok_or_else(|| tera::Error::msg(format!("Invalid date format: {}", date_str)))?;

    let lang = args
        .get("lang")
        .and_then(|v| v.as_str())
        .and_then(Language::from_str)
        .unwrap_or_default();

    let month_name = get_month_name(date.month(), lang);
    let formatted = match lang {
        Language::English => format!("{} {}, {}", month_name, date.day(), date.year()),
        Language::Spanish => format!("{:02} de {} de {}", date.day(), month_name, date.year()),
    };
    Ok(Value::String(formatted))
}

/// Accept both the serialized `DateTime<Utc>` and plain `YYYY-MM-DD` values.
fn parse_date(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return Some(parsed.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|naive| naive.and_utc())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(lang: &str) -> HashMap<String, Value> {
        HashMap::from([("lang".to_string(), Value::String(lang.to_string()))])
    }

    #[test]
    fn formats_spanish_dates() {
        let value = Value::String("2025-08-18T00:00:00+00:00".to_string());
        assert_eq!(
            date_format(&value, &args("es")).expect("formats"),
            Value::String("18 de agosto de 2025".to_string())
        );
    }

    #[test]
    fn formats_english_dates() {
        let value = Value::String("2025-08-18T00:00:00+00:00".to_string());
        assert_eq!(
            date_format(&value, &args("en")).expect("formats"),
            Value::String("August 18, 2025".to_string())
        );
    }

    #[test]
    fn defaults_to_spanish_without_a_language() {
        let value = Value::String("2025-01-05T00:00:00+00:00".to_string());
        assert_eq!(
            date_format(&value, &HashMap::new()).expect("formats"),
            Value::String("05 de enero de 2025".to_string())
        );
    }

    #[test]
    fn accepts_plain_dates() {
        let value = Value::String("2025-08-18".to_string());
        assert!(date_format(&value, &args("en")).is_ok());
    }

    #[test]
    fn reports_invalid_input() {
        assert!(date_format(&Value::String("nope".to_string()), &args("en")).is_err());
        assert!(date_format(&Value::Null, &args("en")).is_err());
    }

    #[test]
    fn names_every_month() {
        assert_eq!(get_month_name(1, Language::Spanish), "enero");
        assert_eq!(get_month_name(12, Language::English), "December");
        assert_eq!(get_month_name(0, Language::English), "Unknown");
        assert_eq!(get_month_name(13, Language::English), "Unknown");
    }
}
