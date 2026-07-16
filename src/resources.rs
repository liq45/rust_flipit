pub struct Resources;
impl Resources {
    pub fn helvetica_bold_cond() -> &'static [u8] { include_bytes!("../res/HelveticaLTStd-BoldCond.ttf") }
    pub fn helvetica_cond() -> &'static [u8] { include_bytes!("../res/HelveticaLTStd-Cond.ttf") }
    pub fn timezone_cities_text() -> &'static str { include_str!("../res/TimeZoneCities.txt") }
    pub fn screen_icon() -> &'static [u8] { include_bytes!("../res/screen.png") }
}
