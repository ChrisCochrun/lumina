// use dirs;
use std::{fs::read_to_string, path::Path};
use tracing::debug;

pub fn count_slides_and_fragments(html_file_path: &Path) -> i32 {
    debug!(path = ?html_file_path, "Starting slide counter");
    // Read the HTML file
    let html_content = read_to_string(html_file_path)
        .expect("Failed to read HTML file");

    // Split HTML content by slide delimiters
    let slide_delimiter = "<section";
    let slide_content: Vec<&str> =
        html_content.split(slide_delimiter).collect();

    // Count slides and fragments
    let num_slides = slide_content.len() - 1;
    let mut num_fragments = 0;

    for slide_html in slide_content.iter().skip(1) {
        let fragments = slide_html.matches("fragment").count();
        num_fragments += fragments;
    }

    let total = num_slides + num_fragments;
    debug!(
        slides = num_slides,
        fragments = num_fragments,
        total = total
    );

    total as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn count_slides_test() {
        let html_file = Path::new("/home/chris/org/lessons/2023_24_1_adam_and_eve_lesson.html");
        assert_eq!(count_slides_and_fragments(html_file), 33);
    }
}
