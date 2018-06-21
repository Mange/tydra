use actions::Page;
use failure::Error;
use tui::widgets::{Paragraph, Widget};
use Term;

pub fn render_list(term: &mut Term, page: &Page) -> Result<(), Error> {
    let size = term.size()?;
    let max_width = size.width as usize;

    let mut text = String::new();

    text.push_str(&format!("== {} ==", page.title));
    for group in &page.groups {
        if let Some(ref title) = group.title {
            text.push_str(&format!("\n\n{}:\n", title));
        } else {
            text.push_str(&format!("\n\n"));
        }

        let mut current_line_length = 0;
        for entry in &group.entries {
            let entry_text = format!("[{}] {} ", entry.shortcut, entry.title);
            if current_line_length + entry_text.len() > max_width {
                text.push('\n');
                current_line_length = 0;
            }
            text.push_str(&entry_text);
        }
    }

    Paragraph::default()
        .wrap(false)
        .text(&text)
        .render(term, &size);
    term.draw().map_err(|e| e.into())
}

pub fn render_columns(term: &mut Term, page: &Page) -> Result<(), Error> {
    // For now, just render like a list anyway
    render_list(term, page)
}
