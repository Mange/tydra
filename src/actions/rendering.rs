use actions::Page;
use failure::Error;

pub fn render_list(page: &Page) -> Result<(), Error> {
    println!("== {} ==", page.title);
    for group in &page.groups {
        if let Some(ref title) = group.title {
            println!("\n{}:", title);
        } else {
            println!("\n");
        }

        for entry in &group.entries {
            println!("[{}] {}", entry.shortcut, entry.title);
        }
    }
    Ok(())
}

pub fn render_columns(page: &Page) -> Result<(), Error> {
    // For now, just render like a list anyway
    println!("(This is supposed to be a column layout)");
    render_list(page)
}
