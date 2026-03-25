pub fn render_portrait_svg(art: &str, colormap: &str) -> String {
    let char_width = 9.6;
    let char_height = 18.0;
    let cols = 48;
    let rows = 32;
    let width = (cols as f64) * char_width;
    let height = (rows as f64) * char_height;

    let bg_color = "#1a1a2e";
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">\n\
         <rect width=\"100%\" height=\"100%\" fill=\"{bg_color}\"/>\n\
         <style>text {{ font-family: 'Courier New', monospace; font-size: 14px; }}</style>\n"
    );

    let art_lines: Vec<&str> = art.split('\n').collect();
    let color_lines: Vec<&str> = colormap.split('\n').collect();

    for (row, (art_line, color_line)) in art_lines.iter().zip(color_lines.iter()).enumerate() {
        let y = (row as f64 + 1.0) * char_height;
        let art_chars: Vec<char> = art_line.chars().collect();
        let color_chars: Vec<char> = color_line.chars().collect();

        for col in 0..art_chars.len().min(cols) {
            let ch = art_chars[col];
            if ch == ' ' {
                continue;
            }
            let color = color_code_to_hex(color_chars.get(col).copied().unwrap_or('.'));
            let x = (col as f64) * char_width;
            let escaped = match ch {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                '/' => "&#x2F;".to_string(),
                _ => ch.to_string(),
            };
            svg.push_str(&format!(
                r#"<text x="{x}" y="{y}" fill="{color}">{escaped}</text>
"#
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn color_code_to_hex(code: char) -> &'static str {
    match code {
        '.' => "#c0c0c0",
        'R' => "#e74c3c",
        'G' => "#2ecc71",
        'B' => "#3498db",
        'C' => "#00bcd4",
        'M' => "#9b59b6",
        'Y' => "#f1c40f",
        'W' => "#ecf0f1",
        'K' => "#2c3e50",
        'O' => "#e67e22",
        _ => "#c0c0c0",
    }
}
