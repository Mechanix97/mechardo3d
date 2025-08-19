use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub route: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(deserialize_with = "deserialize_date")]
    pub date: DateTime<Utc>,
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive_date = NaiveDate::parse_from_str(&s, "%d-%m-%Y").map_err(serde::de::Error::custom)?;
    let datetime = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| serde::de::Error::custom("Invalid date"))?
        .and_utc();
    Ok(datetime)
}
