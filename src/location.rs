use std::collections::HashMap;
use chrono::{NaiveDateTime, TimeZone, Datelike, Timelike};

#[derive(Debug, Clone)]
pub struct Location {
    pub display_name: String,
    pub time_zone_id: String,
    pub current_time: String,
    pub is_dst: bool,
    pub days_difference: i32,
    pub has_error: bool,
}

impl Location {
    pub fn new(time_zone_id: &str, display_name: &str) -> Self {
        Self {
            display_name: display_name.to_string(),
            time_zone_id: time_zone_id.to_string(),
            current_time: String::new(),
            is_dst: false,
            days_difference: 0,
            has_error: false,
        }
    }

    fn tz_offset(tz_id: &str) -> Option<chrono::FixedOffset> {
        let map: HashMap<&str, i32> = [
            ("Dateline Standard Time", -12*60), ("UTC-11", -11*60), ("Hawaiian Standard Time", -10*60),
            ("Alaskan Standard Time", -9*60), ("Pacific Standard Time", -8*60),
            ("US Mountain Standard Time", -7*60), ("Mountain Standard Time", -7*60),
            ("Central Standard Time", -6*60), ("Eastern Standard Time", -5*60),
            ("Atlantic Standard Time", -4*60), ("Newfoundland Standard Time", -3*60-30),
            ("E. South America Standard Time", -3*60), ("Greenland Standard Time", -3*60),
            ("UTC", 0), ("GMT Standard Time", 0), ("Greenwich Standard Time", 0),
            ("W. Europe Standard Time", 60), ("Central Europe Standard Time", 60),
            ("Romance Standard Time", 60), ("W. Central Africa Standard Time", 60),
            ("E. Europe Standard Time", 120), ("South Africa Standard Time", 120),
            ("FLE Standard Time", 120), ("Israel Standard Time", 120),
            ("Turkey Standard Time", 180), ("Russian Standard Time", 180),
            ("E. Africa Standard Time", 180), ("Arabian Standard Time", 240),
            ("Iran Standard Time", 3*60+30), ("Afghanistan Standard Time", 4*60+30),
            ("Pakistan Standard Time", 300), ("India Standard Time", 5*60+30),
            ("Bangladesh Standard Time", 360), ("SE Asia Standard Time", 420),
            ("China Standard Time", 480), ("Singapore Standard Time", 480),
            ("Tokyo Standard Time", 540), ("AUS Eastern Standard Time", 600),
            ("E. Australia Standard Time", 600), ("New Zealand Standard Time", 720),
        ].iter().map(|(k,v)| (*k,*v)).collect();
        map.get(tz_id).copied().and_then(|m| chrono::FixedOffset::east_opt(m * 60))
    }

    pub fn refresh_time(&mut self, now: &NaiveDateTime) {
        if let Some(tz) = Self::tz_offset(&self.time_zone_id) {
            let tz_dt = tz.from_utc_datetime(now);
            self.current_time = tz_dt.format("%H:%M").to_string();
            // days difference approximation
            let now_week = now.iso_week().week();
            let tz_week = tz_dt.iso_week().week();
            self.days_difference = (tz_week as i32 - now_week as i32).signum();
            if tz_dt.hour() < now.hour() && tz_dt.day() < now.day() { self.days_difference = -1; }
            if tz_dt.hour() > now.hour() && tz_dt.day() > now.day() { self.days_difference = 1; }
            self.is_dst = false;
            self.has_error = false;
        } else {
            self.has_error = true;
            self.current_time = String::new();
        }
    }

    pub fn format_time(&self, display_24h: bool, _show_dst: bool) -> String {
        if self.has_error { return "Error".to_string(); }
        let parts: Vec<&str> = self.current_time.split(':').collect();
        let hour: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let min: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let time = if display_24h {
            format!("{:02}:{:02}", hour, min)
        } else {
            let h12 = match hour % 12 { 0 => 12, h => h as i32 };
            format!("{}:{:02} {}", h12, min, if hour >= 12 { "PM" } else { "AM" })
        };
        time
    }
}
