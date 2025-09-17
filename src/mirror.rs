use crate::utils;
use std::error::Error;
use crate::settings::Settings;

pub struct Mirror {
    mirror: Option<String>,
    release: Option<String>
}

impl Mirror {
    pub fn new(mirror: Option<String>, release: Option<String>) -> Self {
        Mirror {
            mirror,
            release
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let sett = Settings::load_or_create();
        
        if self.mirror.clone().unwrap_or_default().is_empty() {
            self.mirror = Some(sett.default_mirror);
        }
        if self.release.clone().unwrap_or_default().is_empty() {
            self.release = Some(sett.release);
        }
        Ok(())
    }

    pub fn get_mirror(&self) -> String {
        format!("{}{}/releases/{}/", self.mirror.as_ref().unwrap(), self.release.as_ref().unwrap(), utils::get_arch())
    }

    pub fn get_repository(&mut self) -> String {
        if self.release == Some("edge".to_string()) {
            return format!("{a}{b}/main\n{a}{b}/community\n{a}{b}/testing", a = self.mirror.as_ref().unwrap(), b = self.release.as_ref().unwrap())
        }
        format!("{a}{b}/main\n{a}{b}/community", a = self.mirror.as_ref().unwrap(), b = self.release.as_ref().unwrap())
    }
}
