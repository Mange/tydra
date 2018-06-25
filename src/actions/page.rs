use super::{Entry, Group, Settings};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    #[serde(default = "Page::default_title")]
    title: String,
    header: Option<String>,
    footer: Option<String>,
    settings: Option<Settings>,
    groups: Vec<Group>,
}

impl Page {
    fn default_title() -> String {
        String::from("Tydra")
    }

    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.groups.iter().flat_map(|group| group.entries.iter())
    }

    pub fn entry_with_shortcut(&self, shortcut: char) -> Option<&Entry> {
        self.all_entries().find(|entry| entry.shortcut == shortcut)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn header(&self) -> Option<&str> {
        self.header.as_ref().map(String::as_ref)
    }

    pub fn footer(&self) -> Option<&str> {
        self.footer.as_ref().map(String::as_ref)
    }

    pub fn settings(&self) -> Option<&Settings> {
        self.settings.as_ref()
    }

    pub fn groups(&self) -> &[Group] {
        self.groups.as_slice()
    }
}
