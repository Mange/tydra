use actions::{Entry, Group, Layout, Page, SettingsAccumulator};
use failure::Error;
use tui::layout::{self, Direction, Rect, Size};
use tui::widgets::{Paragraph, Widget};
use Term;

pub fn render(term: &mut Term, page: &Page, settings: &SettingsAccumulator) -> Result<(), Error> {
    match settings.layout() {
        Layout::List => render_list_layout(term, page, settings),
        Layout::Columns => render_columns_layout(term, page, settings),
    }
}

pub fn render_list_layout(
    term: &mut Term,
    page: &Page,
    settings: &SettingsAccumulator,
) -> Result<(), Error> {
    let size = term.size()?;
    let max_width = size.width as usize;

    let mut text = String::new();

    text.push_str("== ");
    text.push_str(page.title());
    text.push_str(" ==");

    if let Some(ref header) = page.header() {
        text.push_str("\n");
        text.push_str(header);
    }

    for group in page.groups() {
        let settings = settings.with_group(group);

        if let Some(title) = group.title() {
            text.push_str(&format!("\n\n{}:\n", title));
        } else {
            text.push_str("\n\n");
        }

        let mut current_line_length = 0;
        for entry in group.entries() {
            let entry_length = render_entry(entry).len();
            if current_line_length + entry_length > max_width {
                text.push('\n');
                current_line_length = 0;
            }
            text.push_str(&render_entry_color(entry, &settings));
            current_line_length += entry_length;
        }
    }

    if let Some(footer) = page.footer() {
        text.push_str("\n");
        text.push_str(footer);
    }

    Paragraph::default()
        .wrap(true)
        .text(&text)
        .render(term, &size);
    term.draw().map_err(|e| e.into())
}

pub fn render_columns_layout(
    term: &mut Term,
    page: &Page,
    settings: &SettingsAccumulator,
) -> Result<(), Error> {
    let term_size = term.size()?;
    let width = term_size.width as usize;
    let column_widths: Vec<usize> = page
        .groups()
        .iter()
        .map(|group| {
            group
                .entries()
                .iter()
                .map(render_entry)
                .map(|s| s.len())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let required_width = column_widths.iter().sum();

    if width < required_width {
        render_list_layout(term, page, settings)
    } else {
        let header_lines = required_lines_option(page.header(), width);
        let footer_lines = required_lines_option(page.footer(), width);

        layout::Group::default()
            .direction(Direction::Vertical)
            .sizes(&[
                Size::Fixed(1),
                Size::Fixed(header_lines as u16),
                Size::Min(10),
                Size::Fixed(footer_lines as u16),
            ])
            .render(term, &term_size, |t, chunks| {
                render_columns_title(t, chunks[0], page.title(), required_width);
                if let Some(text) = page.header() {
                    render_columns_text(t, chunks[1], &text);
                }
                render_columns(t, chunks[2], &column_widths, page.groups(), &settings);
                if let Some(text) = page.footer() {
                    render_columns_text(t, chunks[3], &text);
                }
            });

        term.draw().map_err(|e| e.into())
    }
}

fn render_columns_text(term: &mut Term, rect: Rect, text: &str) {
    Paragraph::default().text(&text).render(term, &rect);
}

fn render_columns_title(term: &mut Term, rect: Rect, title: &str, width: usize) {
    let centered_title = format!("{title:^width$}", title = title, width = width);
    Paragraph::default()
        .text(&centered_title)
        .render(term, &rect);
}

fn render_columns(
    term: &mut Term,
    rect: Rect,
    column_widths: &[usize],
    groups: &[Group],
    settings: &SettingsAccumulator,
) {
    assert!(column_widths.len() == groups.len());

    let sizes: Vec<Size> = column_widths
        .into_iter()
        .map(|width| Size::Fixed(*width as u16))
        .collect();

    layout::Group::default()
        .direction(Direction::Horizontal)
        .sizes(&sizes)
        .render(term, &rect, |t, chunks| {
            for (chunk, group) in chunks.into_iter().zip(groups.iter()) {
                render_column(t, *chunk, group, &settings);
            }
        });
}

fn render_column(term: &mut Term, rect: Rect, group: &Group, settings: &SettingsAccumulator) {
    let settings = settings.with_group(group);
    let mut text = String::new();

    if let Some(title) = group.title() {
        text.push_str(&format!("{}:\n", title));
    }

    for entry in group.entries() {
        text.push_str(&render_entry_color(entry, &settings));
        text.push('\n');
    }

    Paragraph::default()
        .wrap(true)
        .text(&text)
        .render(term, &rect);
}

fn render_entry(entry: &Entry) -> String {
    format!("[{}] {}  ", entry.shortcut(), entry.title())
}

fn render_entry_color(entry: &Entry, settings: &SettingsAccumulator) -> String {
    let settings = settings.with_entry(entry);
    format!(
        "[{{fg={color} {shortcut}}}] {title}  ",
        shortcut = entry.shortcut(),
        title = entry.title(),
        color = settings.shortcut_color.markup_name()
    )
}

fn required_lines_option(option: Option<&str>, max_width: usize) -> usize {
    match option {
        Some(string) => required_lines(string, max_width),
        None => 0,
    }
}

fn required_lines(string: &str, max_width: usize) -> usize {
    // TODO: Do real calculation with word-wrap (instead of char-wrap), and markup strings removed.
    // This is a very naive implementation right now.
    string
        .split('\n')
        .map(|line| (line.len() / max_width) + 1)
        .sum()
}
