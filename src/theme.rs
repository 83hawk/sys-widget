use relm4::gtk;
use relm4::gtk::{gdk, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::path::PathBuf;

fn find_theme_path(theme: &str) -> Option<PathBuf> {
    let filename = format!("{}.css", theme);

    if let Some(home) = std::env::var_os("HOME") {
        let mut path = PathBuf::from(home);
        path.push(".config/sys-widget/themes");
        path.push(&filename);
        if path.exists() {
            return Some(path);
        }
    }

    let mut system_path = PathBuf::from("/usr/share/sys-widget/themes");
    system_path.push(&filename);
    if system_path.exists() {
        return Some(system_path);
    }

    let mut local_path = PathBuf::from("themes");
    local_path.push(&filename);
    if local_path.exists() {
        return Some(local_path);
    }

    None
}

pub fn load_theme(theme: &str) {
    let provider = CssProvider::new();

    let path = find_theme_path(theme).or_else(|| find_theme_path("default"));

    let Some(path) = path else {
        eprintln!("No valid theme found.");
        return;
    };

    provider.load_from_path(&path);

    println!("Loaded theme from: {}", path.display());

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("No display"),
        &provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
