const PORTRAIT_ROWS: usize = 32;
const PORTRAIT_COLS: usize = 48;
const VALID_COLORS: &[u8] = b".RGBCMYWKO";
const MAX_TAGLINE: usize = 200;
const MAX_COMMENT: usize = 500;
const MAX_STATS_BYTES: usize = 4096;
const MAX_NAME: usize = 32;

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_NAME {
        return Err(format!("Name must be 1-{MAX_NAME} characters"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Name must contain only alphanumeric characters, hyphens, and underscores".into(),
        );
    }
    Ok(())
}

pub fn validate_portrait(portrait: &str) -> Result<(), String> {
    let lines: Vec<&str> = portrait.split('\n').collect();
    if lines.len() != PORTRAIT_ROWS {
        return Err(format!(
            "Portrait must be exactly {PORTRAIT_ROWS} rows, got {}",
            lines.len()
        ));
    }
    for (i, line) in lines.iter().enumerate() {
        if line.len() != PORTRAIT_COLS {
            return Err(format!(
                "Portrait row {i} must be exactly {PORTRAIT_COLS} chars, got {}",
                line.len()
            ));
        }
        if !line.bytes().all(|b| (0x20..=0x7E).contains(&b)) {
            return Err(format!(
                "Portrait row {i} contains non-printable characters"
            ));
        }
    }
    Ok(())
}

pub fn validate_colormap(colormap: &str) -> Result<(), String> {
    let lines: Vec<&str> = colormap.split('\n').collect();
    if lines.len() != PORTRAIT_ROWS {
        return Err(format!(
            "Colormap must be exactly {PORTRAIT_ROWS} rows, got {}",
            lines.len()
        ));
    }
    for (i, line) in lines.iter().enumerate() {
        if line.len() != PORTRAIT_COLS {
            return Err(format!(
                "Colormap row {i} must be exactly {PORTRAIT_COLS} chars, got {}",
                line.len()
            ));
        }
        if !line.bytes().all(|b| VALID_COLORS.contains(&b)) {
            return Err(format!(
                "Colormap row {i} contains invalid color codes. Allowed: . R G B C M Y W K O"
            ));
        }
    }
    Ok(())
}

pub fn validate_tagline(tagline: &str) -> Result<(), String> {
    if tagline.len() > MAX_TAGLINE {
        return Err(format!(
            "Tagline must be at most {MAX_TAGLINE} characters"
        ));
    }
    Ok(())
}

pub fn validate_theme_color(color: &str) -> Result<(), String> {
    if color.len() != 7 || !color.starts_with('#') {
        return Err("Theme color must be #RRGGBB format".into());
    }
    if !color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Theme color must be valid hex (#RRGGBB)".into());
    }
    Ok(())
}

pub fn validate_comment(comment: Option<&str>) -> Result<(), String> {
    if let Some(c) = comment {
        if c.len() > MAX_COMMENT {
            return Err(format!(
                "Comment must be at most {MAX_COMMENT} characters"
            ));
        }
    }
    Ok(())
}

pub fn validate_stats(stats: &str) -> Result<(), String> {
    if stats.len() > MAX_STATS_BYTES {
        return Err(format!(
            "Stats JSON must be at most {MAX_STATS_BYTES} bytes"
        ));
    }
    serde_json::from_str::<serde_json::Value>(stats)
        .map_err(|e| format!("Stats must be valid JSON: {e}"))?;
    Ok(())
}
