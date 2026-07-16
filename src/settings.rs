use crate::display_type::DisplayType;
use crate::location::Location;
use crate::ini_file::IniFile;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ScreenSetting {
    pub screen_number: i32,
    pub device_name: String,
    pub width: i32,
    pub height: i32,
    pub display_type: DisplayType,
    pub locations: Vec<Location>,
}

impl ScreenSetting {
    pub fn new(screen_number: i32, device_name: &str, width: i32, height: i32) -> Self {
        Self { screen_number, device_name: device_name.to_string(), width, height, display_type: DisplayType::CurrentTime, locations: Vec::new() }
    }
    pub fn description(&self) -> String { format!("Screen {} - {} x {}", self.screen_number, self.width, self.height) }
}

#[derive(Debug, Clone)]
pub struct FlipItSettings {
    pub display_24h: bool,
    pub show_dst: bool,
    pub scale: i32,
    pub font_color: u32,      // ARGB, default 0xffb7b7b7
    pub font_alpha: u32,      // 0-255, default 255
    pub screen_settings: Vec<ScreenSetting>,
}

impl FlipItSettings {
    fn settings_folder() -> PathBuf {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
            let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
            format!("{}\\AppData\\Local", home)
        });
        PathBuf::from(local).join("FlipIt")
    }
    fn ini_path() -> PathBuf { Self::settings_folder().join("Settings.ini") }
    fn clean_device_name(name: &str) -> String { name.trim_start_matches('\\').trim_start_matches('.').to_string() }

    pub fn load(screen_devices: &[(i32, String, i32, i32)]) -> Self {
        let ini_path = Self::ini_path();
        let _ = std::fs::create_dir_all(&Self::settings_folder());
        let ini = IniFile::load(&ini_path.to_string_lossy()).unwrap_or_else(|_| IniFile::load("").unwrap());

        let display_24h = ini.get_bool("General", "Display24Hr", true);
        let show_dst = ini.get_bool("General", "ShowDstIndicator", true);
        let scale = ini.get_int("General", "Scale", 100);
        let font_color = ini.get_int("General", "FontColor", 0x00b7b7b7) as u32;
        let font_alpha = ini.get_int("General", "FontAlpha", 255) as u32;

        let mut screen_settings = Vec::new();
        for (num, device_name, w, h) in screen_devices {
            let clean = Self::clean_device_name(device_name);
            let sec = format!("Screen {clean}");
            let loc_sec = format!("Screen {clean} Locations");
            let mut ss = ScreenSetting::new(*num, &clean, *w, *h);
            if ini.section_exists(&sec) {
                let dt = ini.get_int(&sec, "DisplayType", DisplayType::CurrentTime as i32);
                ss.display_type = DisplayType::from_int(dt);
            }
            if ini.section_exists(&loc_sec) {
                for key in ini.get_keys(&loc_sec) {
                    if let Some(tz_id) = ini.get_string(&loc_sec, &key) {
                        if !tz_id.is_empty() { ss.locations.push(Location::new(tz_id, &key)); }
                    }
                }
            }
            screen_settings.push(ss);
        }
        Self { display_24h, show_dst, scale, font_color, font_alpha, screen_settings }
    }

    pub fn save(&self) {
        let ini_path = Self::ini_path();
        let _ = std::fs::create_dir_all(&Self::settings_folder());
        let mut ini = IniFile::load(&ini_path.to_string_lossy()).unwrap_or_else(|_| IniFile::load("").unwrap());
        ini.set_bool("General", "Display24Hr", self.display_24h);
        ini.set_bool("General", "ShowDstIndicator", self.show_dst);
        ini.set_int("General", "Scale", self.scale);
        ini.set_int("General", "FontColor", self.font_color as i32);
        ini.set_int("General", "FontAlpha", self.font_alpha as i32);
        for screen in &self.screen_settings {
            let sec = format!("Screen {}", screen.device_name);
            ini.set_int(&sec, "DisplayType", screen.display_type as i32);
            let loc_sec = format!("Screen {} Locations", screen.device_name);
            if ini.section_exists(&loc_sec) { ini.delete_section(&loc_sec); }
            for loc in &screen.locations {
                ini.set_string(&loc_sec, &loc.display_name, &loc.time_zone_id);
            }
        }
        let _ = ini.save();
    }

    pub fn default_locations() -> Vec<Location> {
        vec![
            Location::new("Pacific Standard Time", "Los Angeles"),
            Location::new("Eastern Standard Time", "New York (EST)"),
            Location::new("E. Australia Standard Time", "Brisbane"),
            Location::new("New Zealand Standard Time", "Wellington"),
            Location::new("AUS Eastern Standard Time", "Melbourne"),
            Location::new("UTC", "UTC"),
            Location::new("GMT Standard Time", "London"),
            Location::new("Romance Standard Time", "Paris"),
            Location::new("Hawaiian Standard Time", "Hawaii"),
        ]
    }
}
