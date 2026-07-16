use std::collections::HashMap;
use crate::location::Location;

pub fn load_entries() -> Vec<(String, Vec<String>)> {
    let text = crate::resources::Resources::timezone_cities_text();
    let mut result = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with(';') { continue; }
        if let Some(eq) = t.find('=') {
            let tz = t[..eq].trim().to_string();
            let cities: Vec<String> = t[eq+1..].split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty() && c != "PST8PDT" && !(c.starts_with("GMT") && c.len() > 3))
                .collect();
            result.push((tz, cities));
        }
    }
    result
}

pub fn build_available_locations() -> Vec<Location> {
    let mut v = Vec::new();
    for (tz, cities) in load_entries() {
        for city in cities { v.push(Location::new(&tz, &city)); }
    }
    v
}

pub fn build_city_to_tz_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (tz, cities) in load_entries() {
        for city in cities { map.insert(city.to_lowercase(), tz.clone()); }
    }
    map
}
