use crate::actions::{ActionFile, Command, Entry, Return, RunMode};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Fail)]
pub enum ValidationError {
    #[fail(
        display = "Found reference to an unknown page: {}",
        page_name
    )]
    UnknownPage { page_name: String },
    #[fail(display = "Found page with no entries: {}", page_name)]
    EmptyPage { page_name: String },
    #[fail(
        display = "Specified root page does not exist: {}",
        root_name
    )]
    NoRoot { root_name: String },
    #[fail(
        display = "Page {} has a duplicated shortcut: {} ({})",
        page_name,
        shortcut,
        title
    )]
    DuplicatedShortcut {
        page_name: String,
        shortcut: char,
        title: String,
    },
    #[fail(
        display = "Entry cannot return and exec at the same time; exec will replace tydra process (page {}, shortcut {}).",
        page_name,
        shortcut
    )]
    ExecWithReturn { page_name: String, shortcut: char },
    #[fail(
        display = "Entry cannot exec without a command (page {}, shortcut {}).",
        page_name,
        shortcut
    )]
    ExecWithoutCommand { page_name: String, shortcut: char },
}

pub fn validate(actions: &ActionFile, root_name: &str) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();

    if !actions.has_page(root_name) {
        errors.push(ValidationError::NoRoot {
            root_name: root_name.into(),
        });
    }

    for (page, page_name) in actions.pages_with_names() {
        let mut seen_shortcuts = HashSet::new();

        if page.all_entries().next().is_none() {
            errors.push(ValidationError::EmptyPage {
                page_name: page_name.to_owned(),
            });
        }

        for entry in page.all_entries() {
            validate_shortcut_duplicates(&mut errors, entry, &mut seen_shortcuts, page_name);
            validate_return_link(&mut errors, entry, actions);
            validate_mode(&mut errors, entry, page_name);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_shortcut_duplicates(
    errors: &mut Vec<ValidationError>,
    entry: &Entry,
    seen_shortcuts: &mut HashSet<char>,
    page_name: &str,
) {
    let shortcut = entry.shortcut();
    if !seen_shortcuts.insert(shortcut) {
        errors.push(ValidationError::DuplicatedShortcut {
            page_name: page_name.to_owned(),
            shortcut,
            title: entry.title().into(),
        });
    }
}

fn validate_return_link(errors: &mut Vec<ValidationError>, entry: &Entry, actions: &ActionFile) {
    if let Return::OtherPage(page_name) = entry.return_to() {
        if !actions.has_page(page_name) {
            errors.push(ValidationError::UnknownPage {
                page_name: page_name.clone(),
            });
        }
    }
}

fn validate_mode(errors: &mut Vec<ValidationError>, entry: &Entry, page_name: &str) {
    if entry.runner_mode() == RunMode::Exec {
        match entry.return_to() {
            Return::SamePage | Return::OtherPage(_) => {
                errors.push(ValidationError::ExecWithReturn {
                    page_name: page_name.to_owned(),
                    shortcut: entry.shortcut(),
                });
            }
            Return::Quit => {}
        }
        match entry.command() {
            Command::None => errors.push(ValidationError::ExecWithoutCommand {
                page_name: page_name.to_owned(),
                shortcut: entry.shortcut(),
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate serde_yaml;

    #[test]
    fn it_validates_missing_pages() {
        let actions: ActionFile =
            serde_yaml::from_str(include_str!("../../tests/fixtures/unknown_page.yml")).unwrap();
        let errors = validate(&actions, "root").unwrap_err();

        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            ValidationError::UnknownPage {
                page_name: "speling_error".into(),
            },
        );
        assert_eq!(
            errors[1],
            ValidationError::UnknownPage {
                page_name: "does_not_exist".into(),
            },
        );
    }

    #[test]
    fn it_validates_empty_pages() {
        let actions: ActionFile = serde_yaml::from_str(
            r#"
pages:
  root:
    groups:
      - entries:
          - shortcut: a
            title: Working
  this_page_is_empty:
    groups:
        - entries: []"#,
        ).unwrap();

        let errors = validate(&actions, "root").unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ValidationError::EmptyPage {
                page_name: "this_page_is_empty".into(),
            },
        );
    }

    #[test]
    fn it_validates_no_root_page() {
        let actions: ActionFile = serde_yaml::from_str(
            r#"
pages:
  potato:
    groups:
      - entries:
          - shortcut: a
            title: Working"#,
        ).unwrap();

        let errors = validate(&actions, "horseradish").unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ValidationError::NoRoot {
                root_name: String::from("horseradish")
            }
        );
    }

    #[test]
    fn it_validates_duplicated_keys() {
        let actions: ActionFile = serde_yaml::from_str(
            r#"
pages:
  root:
    groups:
      - entries:
          - shortcut: a
            title: This is fine
      - entries:
          - shortcut: b
            title: This is fine
  bad_page:
    groups:
      - entries:
          - shortcut: a
            title: First one
      - entries:
          - shortcut: a
            title: Duplicated shortcut"#,
        ).unwrap();

        let errors = validate(&actions, "root").unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ValidationError::DuplicatedShortcut {
                page_name: "bad_page".into(),
                shortcut: 'a',
                title: "Duplicated shortcut".into(),
            }
        );
    }

    #[test]
    fn it_validates_no_exec_with_return() {
        let actions: ActionFile = serde_yaml::from_str(
            r#"
pages:
  root:
    groups:
      - entries:
          - shortcut: a
            title: This is fine
            command: /bin/true
            mode: exec
          - shortcut: b
            title: This makes no sense
            command: /bin/true
            mode: exec
            return: true
          - shortcut: c
            title: This neither
            command: /bin/true
            mode: exec
            return: root"#,
        ).unwrap();

        let errors = validate(&actions, "root").unwrap_err();

        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            ValidationError::ExecWithReturn {
                page_name: "root".into(),
                shortcut: 'b',
            }
        );
        assert_eq!(
            errors[1],
            ValidationError::ExecWithReturn {
                page_name: "root".into(),
                shortcut: 'c',
            }
        );
    }

    #[test]
    fn it_validates_exec_with_no_command() {
        let actions: ActionFile = serde_yaml::from_str(
            r#"
pages:
  root:
    groups:
      - entries:
          - shortcut: a
            title: This is fine
            mode: exec
            command: /bin/true
          - shortcut: b
            title: This makes no sense (explicitly no command)
            mode: exec
            command: null
          - shortcut: c
            title: This neither (no command mentioned)
            mode: exec"#,
        ).unwrap();

        let errors = validate(&actions, "root").unwrap_err();

        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            ValidationError::ExecWithoutCommand {
                page_name: "root".into(),
                shortcut: 'b',
            }
        );
        assert_eq!(
            errors[1],
            ValidationError::ExecWithoutCommand {
                page_name: "root".into(),
                shortcut: 'c',
            }
        );
    }
}
