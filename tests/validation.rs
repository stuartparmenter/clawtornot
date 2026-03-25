use clawtornot::validation::*;

#[test]
fn valid_name() {
    assert!(validate_name("xX_ClawDaddy_Xx").is_ok());
    assert!(validate_name("a").is_ok());
    assert!(validate_name("agent-007").is_ok());
}

#[test]
fn invalid_names() {
    assert!(validate_name("").is_err());
    assert!(validate_name(&"a".repeat(33)).is_err());
    assert!(validate_name("has spaces").is_err());
}

#[test]
fn valid_portrait() {
    let line = " ".repeat(48);
    let portrait = (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n");
    assert!(validate_portrait(&portrait).is_ok());
}

#[test]
fn portrait_wrong_dimensions() {
    let line = " ".repeat(47);
    let portrait = (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n");
    assert!(validate_portrait(&portrait).is_err());
}

#[test]
fn valid_colormap() {
    let line = ".".repeat(48);
    let colormap = (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n");
    assert!(validate_colormap(&colormap).is_ok());
}

#[test]
fn colormap_invalid_code() {
    let line = "X".repeat(48);
    let colormap = (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n");
    assert!(validate_colormap(&colormap).is_err());
}

#[test]
fn valid_tagline() {
    assert!(validate_tagline("I am the alpha lobster.").is_ok());
    assert!(validate_tagline(&"x".repeat(200)).is_ok());
}

#[test]
fn tagline_too_long() {
    assert!(validate_tagline(&"x".repeat(201)).is_err());
}

#[test]
fn valid_theme_color() {
    assert!(validate_theme_color("#ff6b6b").is_ok());
    assert!(validate_theme_color("#AABBCC").is_ok());
}

#[test]
fn invalid_theme_color() {
    assert!(validate_theme_color("ff6b6b").is_err());
    assert!(validate_theme_color("#gggggg").is_err());
    assert!(validate_theme_color("#fff").is_err());
}

#[test]
fn valid_comment() {
    assert!(validate_comment(Some("sick burn")).is_ok());
    assert!(validate_comment(None).is_ok());
}

#[test]
fn comment_too_long() {
    assert!(validate_comment(Some(&"x".repeat(501))).is_err());
}

#[test]
fn valid_stats_json() {
    assert!(validate_stats(r#"{"hardware":"Pi 5"}"#).is_ok());
}

#[test]
fn stats_too_large() {
    let big = format!(r#"{{"data":"{}"}}"#, "x".repeat(4096));
    assert!(validate_stats(&big).is_err());
}
