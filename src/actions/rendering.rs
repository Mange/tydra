use actions::{Entry, Group, Page};
use failure::Error;
use tui::layout::{self, Direction, Rect, Size};
use tui::widgets::{Paragraph, Widget};
use Term;

pub fn render_list_layout(term: &mut Term, page: &Page) -> Result<(), Error> {
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
            let entry_text = render_entry(entry);
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

pub fn render_columns_layout(term: &mut Term, page: &Page) -> Result<(), Error> {
    let term_size = term.size()?;
    let width = term_size.width as usize;
    let column_widths: Vec<usize> = page.groups
        .iter()
        .map(|group| {
            group
                .entries
                .iter()
                .map(render_entry)
                .map(|s| s.len())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let required_width = column_widths.iter().sum();

    if width < required_width {
        render_list_layout(term, page)
    } else {
        layout::Group::default()
            .direction(Direction::Vertical)
            .sizes(&[Size::Fixed(1), Size::Min(10)])
            .render(term, &term_size, |t, chunks| {
                render_columns_title(t, &chunks[0], &page.title, required_width);
                render_columns(t, &chunks[1], &column_widths, &page.groups);
            });

        term.draw().map_err(|e| e.into())
    }
}

fn render_columns_title(term: &mut Term, rect: &Rect, title: &str, width: usize) {
    let centered_title = format!("{title:^width$}", title = title, width = width);
    Paragraph::default()
        .text(&centered_title)
        .render(term, rect);
}

fn render_columns(term: &mut Term, rect: &Rect, column_widths: &[usize], groups: &[Group]) {
    assert!(column_widths.len() == groups.len());

    let sizes: Vec<Size> = column_widths
        .into_iter()
        .map(|width| Size::Fixed(*width as u16))
        .collect();

    layout::Group::default()
        .direction(Direction::Horizontal)
        .sizes(&sizes)
        .render(term, rect, |t, chunks| {
            for (chunk, group) in chunks.into_iter().zip(groups.iter()) {
                render_column(t, chunk, group);
            }
        });
}

fn render_column(term: &mut Term, rect: &Rect, group: &Group) {
    let mut text = String::new();

    if let Some(ref title) = group.title {
        text.push_str(&format!("{}:\n", title));
    }

    for entry in &group.entries {
        text.push_str(&render_entry(entry));
        text.push('\n');
    }

    Paragraph::default()
        .wrap(false)
        .text(&text)
        .render(term, rect);
}

fn render_entry(entry: &Entry) -> String {
    format!("[{}] {} ", entry.shortcut(), entry.title())
}
