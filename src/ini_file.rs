use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Simple INI file reader/writer
pub struct IniFile {
    sections: BTreeMap<String, BTreeMap<String, String>>,
    path: String,
}

impl IniFile {
    pub fn load(path: &str) -> io::Result<Self> {
        let p = Path::new(path);
        let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        if !p.exists() {
            return Ok(Self { sections, path: path.to_string() });
        }
        let content = fs::read_to_string(path)?;
        let mut current_section = "ROOT".to_string();
        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with(';') || t.starts_with('#') { continue; }
            if t.starts_with('[') && t.ends_with(']') {
                current_section = t[1..t.len()-1].to_string();
            } else if let Some(eq) = t.find('=') {
                let key = t[..eq].trim().to_string();
                let val = t[eq+1..].trim().to_string();
                sections.entry(current_section.clone()).or_default().insert(key, val);
            }
        }
        Ok(Self { sections, path: path.to_string() })
    }

    pub fn section_exists(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    pub fn get_keys(&self, section: &str) -> Vec<String> {
        self.sections.get(section).map(|m| m.keys().cloned().collect()).unwrap_or_default()
    }

    pub fn get_string(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.get(section).and_then(|m| m.get(key)).map(|s| s.as_str())
    }

    pub fn get_int(&self, section: &str, key: &str, default: i32) -> i32 {
        self.get_string(section, key).and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    pub fn get_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.get_int(section, key, if default { 1 } else { 0 }) == 1
    }

    pub fn set_string(&mut self, section: &str, key: &str, value: &str) {
        self.sections.entry(section.to_string()).or_default().insert(key.to_string(), value.to_string());
    }

    pub fn set_int(&mut self, section: &str, key: &str, value: i32) {
        self.set_string(section, key, &value.to_string());
    }

    pub fn set_bool(&mut self, section: &str, key: &str, value: bool) {
        self.set_string(section, key, if value { "1" } else { "0" });
    }

    pub fn delete_key(&mut self, section: &str, key: &str) {
        if let Some(m) = self.sections.get_mut(section) { m.remove(key); }
    }

    pub fn delete_section(&mut self, section: &str) {
        self.sections.remove(section);
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = Path::new(&self.path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = Vec::new();
        for (section, entries) in &self.sections {
            if entries.is_empty() { continue; }
            writeln!(out, "[{section}]")?;
            for (key, value) in entries {
                writeln!(out, "{key}={value}")?;
            }
            writeln!(out)?;
        }
        fs::write(&self.path, out)?;
        Ok(())
    }
}
