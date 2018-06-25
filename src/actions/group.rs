use super::{Entry, Settings};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    title: Option<String>,
    settings: Option<Settings>,
    entries: Vec<Entry>,
}

impl Group {
    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(String::as_ref)
    }

    pub fn settings(&self) -> Option<&Settings> {
        self.settings.as_ref()
    }

    pub fn entries(&self) -> &[Entry] {
        self.entries.as_slice()
    }
}
