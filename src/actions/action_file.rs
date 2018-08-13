use super::{validator, Page, Settings, SettingsAccumulator, ValidationError};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionFile {
    #[serde(rename = "global", default = "Settings::default")]
    global_settings: Settings,
    pages: BTreeMap<String, Page>, // BTreeMap so order is preserved; helps with validation logic, etc.
}

impl ActionFile {
    pub fn pages_with_names(&self) -> impl Iterator<Item = (&Page, &str)> {
        self.pages.iter().map(|(name, page)| (page, name.as_ref()))
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        validator::validate(self)
    }

    pub fn has_page(&self, page_name: &str) -> bool {
        self.pages.contains_key(page_name)
    }

    pub fn get_page(&self, page_name: &str) -> &Page {
        &self.pages[page_name]
    }

    pub fn settings_accumulator(&self) -> SettingsAccumulator {
        SettingsAccumulator::from(&self.global_settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate serde_yaml;

    #[test]
    fn it_loads_minimal_yaml() {
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/minimal.yml")).unwrap();
        actions.validate().unwrap();
    }

    #[test]
    fn it_loads_complex_yaml() {
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/complex.yml")).unwrap();
        actions.validate().unwrap();
    }
}
