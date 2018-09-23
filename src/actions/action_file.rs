use super::{validator, Page, Settings, SettingsAccumulator, ValidationError};
use std::collections::BTreeMap;
use AppOptions;

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

    pub fn validate(&self, options: &AppOptions) -> Result<(), Vec<ValidationError>> {
        validator::validate(self, &options.start_page)
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

    fn default_options() -> AppOptions {
        AppOptions {
            filename: String::from("/dev/null"),
            ignore_exit_status: false,
            start_page: String::from("root"),
            validate: false,
        }
    }

    #[test]
    fn it_loads_minimal_yaml() {
        let options = default_options();
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/minimal.yml")).unwrap();
        actions.validate(&options).unwrap();
    }

    #[test]
    fn it_loads_complex_yaml() {
        let options = default_options();
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/complex.yml")).unwrap();
        actions.validate(&options).unwrap();
    }
}
