use actions::ActionFile;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Fail)]
pub enum ValidationError {
    #[fail(display = "Found reference to an unknown page: {}", page_name)]
    UnknownPage { page_name: String },
    #[fail(display = "Found page with no entries: {}", page_name)]
    EmptyPage { page_name: String },
    #[fail(display = "There is no root page. A root page must be specified.")]
    NoRoot,
    #[fail(display = "Page {} has a duplicated shortcut: {} ({})", page_name, shortcut, title)]
    DuplicatedShortcut {
        page_name: String,
        shortcut: char,
        title: String,
    },
}

pub fn validate(actions: &ActionFile) -> Result<(), Vec<ValidationError>> {
    let mut errors: Vec<ValidationError> = Vec::new();

    if !actions.has_page("root") {
        errors.push(ValidationError::NoRoot);
    }

    for (page, page_name) in actions.pages_with_names() {
        let mut seen_shortcuts = HashSet::new();

        if page.all_entries().next().is_none() {
            errors.push(ValidationError::EmptyPage {
                page_name: page_name.to_owned(),
            });
        }

        for entry in page.all_entries() {
            let shortcut = entry.shortcut();
            if !seen_shortcuts.insert(shortcut) {
                errors.push(ValidationError::DuplicatedShortcut {
                    page_name: page_name.to_owned(),
                    shortcut: shortcut,
                    title: entry.title().into(),
                });
            }

            match entry.return_to.as_ref().map(String::as_ref) {
                Some("quit") => {}
                Some(page_name) if !actions.has_page(page_name) => {
                    errors.push(ValidationError::UnknownPage {
                        page_name: page_name.into(),
                    });
                }
                _ => {}
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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
        let errors = actions.validate().unwrap_err();

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
            return: quit
  this_page_is_empty:
    groups:
        - entries: []"#,
        ).unwrap();

        let errors = actions.validate().unwrap_err();

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
  foo_page:
    groups:
      - entries:
          - shortcut: a
            title: Working
            return: quit"#,
        ).unwrap();

        let errors = actions.validate().unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], ValidationError::NoRoot);
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
            return: quit
      - entries:
          - shortcut: b
            title: This is fine
            return: quit
  bad_page:
    groups:
      - entries:
          - shortcut: a
            title: First one
            return: quit
      - entries:
          - shortcut: a
            title: Duplicated shortcut
            return: quit"#,
        ).unwrap();

        let errors = actions.validate().unwrap_err();

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
}
