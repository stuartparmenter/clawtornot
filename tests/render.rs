use clawtornot::render::svg::render_portrait_svg;

#[test]
fn renders_basic_svg() {
    let mut lines: Vec<String> = (0..32).map(|_| " ".repeat(48)).collect();
    lines[16].replace_range(24..25, "X");
    let art = lines.join("\n");

    let mut clines: Vec<String> = (0..32).map(|_| ".".repeat(48)).collect();
    clines[16].replace_range(24..25, "R");
    let colormap = clines.join("\n");

    let svg = render_portrait_svg(&art, &colormap);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("X"));
    assert!(svg.contains("#e74c3c")); // red
}

#[test]
fn all_color_codes_produce_valid_svg() {
    let art = (0..32).map(|_| " ".repeat(48)).collect::<Vec<_>>().join("\n");
    let cline: String = ".RGBCMYWKO"
        .chars()
        .chain(std::iter::repeat_with(|| '.'))
        .take(48)
        .collect();
    let colormap = (0..32).map(|_| cline.as_str()).collect::<Vec<_>>().join("\n");

    let svg = render_portrait_svg(&art, &colormap);
    assert!(svg.contains("<svg"));
}
