//! `style` prop applier.
//!
//! React-shaped: `<div style={{ padding: 17, color: "#a855f7" }} />`.
//! Numbers default to px (React Native convention). Strings handle `%`,
//! `#rrggbb`, `rgb()`, and a few named colors. Keys are camelCase.

use std::collections::HashMap;

use gpui::{prelude::*, px, relative, rgb, rgba, Div, Hsla};
use serde_json::Value as JsonValue;

/// Walk the style map and apply each known key. Unknown keys are dropped.
pub fn apply(mut d: Div, style: &HashMap<String, JsonValue>) -> Div {
    for (key, value) in style {
        d = apply_one(d, key.as_str(), value);
    }
    d
}

fn apply_one(d: Div, key: &str, value: &JsonValue) -> Div {
    match key {
        "padding" => with_length(d, value, |d, l| d.p(l)),
        "paddingTop" => with_length(d, value, |d, l| d.pt(l)),
        "paddingRight" => with_length(d, value, |d, l| d.pr(l)),
        "paddingBottom" => with_length(d, value, |d, l| d.pb(l)),
        "paddingLeft" => with_length(d, value, |d, l| d.pl(l)),
        "paddingX" | "paddingHorizontal" => with_length(d, value, |d, l| d.px(l)),
        "paddingY" | "paddingVertical" => with_length(d, value, |d, l| d.py(l)),

        "margin" => with_length(d, value, |d, l| d.m(l)),
        "marginTop" => with_length(d, value, |d, l| d.mt(l)),
        "marginRight" => with_length(d, value, |d, l| d.mr(l)),
        "marginBottom" => with_length(d, value, |d, l| d.mb(l)),
        "marginLeft" => with_length(d, value, |d, l| d.ml(l)),
        "marginX" | "marginHorizontal" => with_length(d, value, |d, l| d.mx(l)),
        "marginY" | "marginVertical" => with_length(d, value, |d, l| d.my(l)),

        "width" => with_length(d, value, |d, l| d.w(l)),
        "height" => with_length(d, value, |d, l| d.h(l)),
        "minWidth" => with_length(d, value, |d, l| d.min_w(l)),
        "minHeight" => with_length(d, value, |d, l| d.min_h(l)),
        "maxWidth" => with_length(d, value, |d, l| d.max_w(l)),
        "maxHeight" => with_length(d, value, |d, l| d.max_h(l)),

        "gap" => with_length(d, value, |d, l| d.gap(l)),

        "borderRadius" => with_length_abs(d, value, |d, l| d.rounded(l)),
        "borderTopLeftRadius" => with_length_abs(d, value, |d, l| d.rounded_tl(l)),
        "borderTopRightRadius" => with_length_abs(d, value, |d, l| d.rounded_tr(l)),
        "borderBottomLeftRadius" => with_length_abs(d, value, |d, l| d.rounded_bl(l)),
        "borderBottomRightRadius" => with_length_abs(d, value, |d, l| d.rounded_br(l)),

        "color" => with_color(d, value, |d, c| d.text_color(c)),
        "backgroundColor" => with_color(d, value, |d, c| d.bg(c)),
        "borderColor" => with_color(d, value, |d, c| d.border_color(c)),

        "opacity" => with_f32(d, value, |d, n| d.opacity(n)),

        // Quietly ignore unknown keys — covers the long tail.
        _ => d,
    }
}

fn with_length(d: Div, value: &JsonValue, f: impl FnOnce(Div, gpui::DefiniteLength) -> Div) -> Div {
    if let Some(n) = value.as_f64() {
        return f(d, px(n as f32).into());
    }
    if let Some(s) = value.as_str() {
        if let Some(pct) = s.strip_suffix('%') {
            if let Ok(p) = pct.parse::<f32>() {
                return f(d, relative(p / 100.0));
            }
        }
        if let Some(pxs) = s.strip_suffix("px") {
            if let Ok(n) = pxs.parse::<f32>() {
                return f(d, px(n).into());
            }
        }
        if let Ok(n) = s.parse::<f32>() {
            return f(d, px(n).into());
        }
    }
    d
}

fn with_length_abs(
    d: Div,
    value: &JsonValue,
    f: impl FnOnce(Div, gpui::AbsoluteLength) -> Div,
) -> Div {
    if let Some(n) = value.as_f64() {
        return f(d, px(n as f32).into());
    }
    if let Some(s) = value.as_str() {
        if let Some(pxs) = s.strip_suffix("px") {
            if let Ok(n) = pxs.parse::<f32>() {
                return f(d, px(n).into());
            }
        }
        if let Ok(n) = s.parse::<f32>() {
            return f(d, px(n).into());
        }
    }
    d
}

fn with_color(d: Div, value: &JsonValue, f: impl FnOnce(Div, Hsla) -> Div) -> Div {
    let Some(s) = value.as_str() else {
        return d;
    };
    if let Some(c) = parse_color(s) {
        f(d, c)
    } else {
        d
    }
}

fn with_f32(d: Div, value: &JsonValue, f: impl FnOnce(Div, f32) -> Div) -> Div {
    if let Some(n) = value.as_f64() {
        return f(d, n as f32);
    }
    d
}

/// Parse a CSS-ish color into `Hsla`. Supports `#rrggbb`, `#rrggbbaa`,
/// `rgb(r,g,b)`, `rgba(r,g,b,a)`, and a few named colors.
fn parse_color(s: &str) -> Option<Hsla> {
    let s = s.trim();
    if let Some(named) = named_color(s) {
        return Some(rgb(named).into());
    }
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    if let Some(rest) = s.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        return parse_rgb_args(rest, false);
    }
    if let Some(rest) = s.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
        return parse_rgb_args(rest, true);
    }
    None
}

fn parse_hex(hex: &str) -> Option<Hsla> {
    match hex.len() {
        6 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(rgb(v).into())
        }
        8 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(rgba(v).into())
        }
        3 => {
            let mut full = String::with_capacity(6);
            for ch in hex.chars() {
                full.push(ch);
                full.push(ch);
            }
            let v = u32::from_str_radix(&full, 16).ok()?;
            Some(rgb(v).into())
        }
        _ => None,
    }
}

fn parse_rgb_args(args: &str, with_alpha: bool) -> Option<Hsla> {
    let parts: Vec<&str> = args.split(',').map(|p| p.trim()).collect();
    if !with_alpha && parts.len() != 3 {
        return None;
    }
    if with_alpha && parts.len() != 4 {
        return None;
    }
    let r: u8 = parts[0].parse().ok()?;
    let g: u8 = parts[1].parse().ok()?;
    let b: u8 = parts[2].parse().ok()?;
    let v = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    if with_alpha {
        let a: f32 = parts[3].parse().ok()?;
        let av = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
        Some(rgba((v << 8) | av).into())
    } else {
        Some(rgb(v).into())
    }
}

fn named_color(name: &str) -> Option<u32> {
    Some(match name.to_ascii_lowercase().as_str() {
        "white" => 0xFFFFFF,
        "black" => 0x000000,
        "red" => 0xFF0000,
        "green" => 0x008000,
        "blue" => 0x0000FF,
        "transparent" => return None,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_color;

    #[test]
    fn parses_hex() {
        assert!(parse_color("#a855f7").is_some());
        assert!(parse_color("#fff").is_some());
        assert!(parse_color("not-a-color").is_none());
    }

    #[test]
    fn parses_rgb() {
        assert!(parse_color("rgb(255, 0, 0)").is_some());
        assert!(parse_color("rgba(255, 0, 0, 0.5)").is_some());
    }
}
