//! Tailwind class subset.
//!
//! `parse(&[class])` returns a list of `StyleOp` closures. Each closure takes
//! a GPUI `Div` and returns it with one fluent method applied. The render
//! walk folds them in order. `style` prop wins over classes — applied last.

use gpui::{prelude::*, px, rgb, Div};

/// One style mutation. Boxed because GPUI's fluent methods return new `Div`
/// values; we accumulate these in a Vec at compile time of the render walk.
pub type StyleOp = Box<dyn FnOnce(Div) -> Div>;

/// Parse a class list into a sequence of style ops. Unknown classes are
/// dropped silently — the caller can warn via `__host_log` if desired.
pub fn parse(classes: &[String]) -> Vec<StyleOp> {
    let mut ops: Vec<StyleOp> = Vec::with_capacity(classes.len());
    for raw in classes {
        if let Some(op) = parse_one(raw) {
            ops.push(op);
        }
    }
    ops
}

fn parse_one(class: &str) -> Option<StyleOp> {
    // Layout
    match class {
        "flex" => return Some(Box::new(|d: Div| d.flex())),
        "flex-col" => return Some(Box::new(|d: Div| d.flex_col())),
        "flex-row" => return Some(Box::new(|d: Div| d.flex_row())),
        "flex-1" => return Some(Box::new(|d: Div| d.flex_1())),
        "items-center" => return Some(Box::new(|d: Div| d.items_center())),
        "items-start" => return Some(Box::new(|d: Div| d.items_start())),
        "items-end" => return Some(Box::new(|d: Div| d.items_end())),
        "justify-center" => return Some(Box::new(|d: Div| d.justify_center())),
        "justify-between" => return Some(Box::new(|d: Div| d.justify_between())),
        "justify-start" => return Some(Box::new(|d: Div| d.justify_start())),
        "justify-end" => return Some(Box::new(|d: Div| d.justify_end())),
        "rounded" => return Some(Box::new(|d: Div| d.rounded(px(4.0)))),
        "rounded-md" => return Some(Box::new(|d: Div| d.rounded(px(6.0)))),
        "rounded-lg" => return Some(Box::new(|d: Div| d.rounded(px(8.0)))),
        "rounded-full" => return Some(Box::new(|d: Div| d.rounded(px(9999.0)))),
        "border" => return Some(Box::new(|d: Div| d.border_1())),
        "w-full" => return Some(Box::new(|d: Div| d.w_full())),
        "h-full" => return Some(Box::new(|d: Div| d.h_full())),
        "size-full" => return Some(Box::new(|d: Div| d.size_full())),
        "text-xs" => return Some(Box::new(|d: Div| d.text_xs())),
        "text-sm" => return Some(Box::new(|d: Div| d.text_sm())),
        "text-base" => return Some(Box::new(|d: Div| d.text_base())),
        "text-lg" => return Some(Box::new(|d: Div| d.text_lg())),
        "text-xl" => return Some(Box::new(|d: Div| d.text_xl())),
        "text-2xl" => return Some(Box::new(|d: Div| d.text_2xl())),
        "text-3xl" => return Some(Box::new(|d: Div| d.text_3xl())),
        // GPUI has no text_4xl/5xl helpers; use the Tailwind px values.
        "text-4xl" => return Some(Box::new(|d: Div| d.text_size(px(36.0)))),
        "text-5xl" => return Some(Box::new(|d: Div| d.text_size(px(48.0)))),
        "font-semibold" => {
            return Some(Box::new(|d: Div| d.font_weight(gpui::FontWeight::SEMIBOLD)))
        }
        "font-bold" => return Some(Box::new(|d: Div| d.font_weight(gpui::FontWeight::BOLD))),
        "line-through" => return Some(Box::new(|d: Div| d.line_through())),
        _ => {}
    }

    // Spacing: gap, p, px, py, pt, pr, pb, pl, m, mx, my, mt, mr, mb, ml
    if let Some(rest) = class.strip_prefix("gap-") {
        let n = rest.parse::<f32>().ok()?;
        return Some(Box::new(move |d: Div| d.gap(px(n * 4.0))));
    }
    if let Some(op) = parse_spacing(class, "p", |d, v| d.p(v))
        .or_else(|| parse_spacing(class, "px", |d, v| d.px(v)))
        .or_else(|| parse_spacing(class, "py", |d, v| d.py(v)))
        .or_else(|| parse_spacing(class, "pt", |d, v| d.pt(v)))
        .or_else(|| parse_spacing(class, "pr", |d, v| d.pr(v)))
        .or_else(|| parse_spacing(class, "pb", |d, v| d.pb(v)))
        .or_else(|| parse_spacing(class, "pl", |d, v| d.pl(v)))
        .or_else(|| parse_spacing(class, "m", |d, v| d.m(v)))
        .or_else(|| parse_spacing(class, "mx", |d, v| d.mx(v)))
        .or_else(|| parse_spacing(class, "my", |d, v| d.my(v)))
        .or_else(|| parse_spacing(class, "mt", |d, v| d.mt(v)))
        .or_else(|| parse_spacing(class, "mr", |d, v| d.mr(v)))
        .or_else(|| parse_spacing(class, "mb", |d, v| d.mb(v)))
        .or_else(|| parse_spacing(class, "ml", |d, v| d.ml(v)))
    {
        return Some(op);
    }

    // Width / height as numeric units (`w-4`, `h-12`).
    if let Some(rest) = class.strip_prefix("w-") {
        if let Ok(n) = rest.parse::<f32>() {
            return Some(Box::new(move |d: Div| d.w(px(n * 4.0))));
        }
    }
    if let Some(rest) = class.strip_prefix("h-") {
        if let Ok(n) = rest.parse::<f32>() {
            return Some(Box::new(move |d: Div| d.h(px(n * 4.0))));
        }
    }

    // Background and text colors via the palette table.
    if let Some(rest) = class.strip_prefix("bg-") {
        if let Some(c) = palette(rest) {
            return Some(Box::new(move |d: Div| d.bg(rgb(c))));
        }
    }
    if let Some(rest) = class.strip_prefix("text-") {
        if let Some(c) = palette(rest) {
            return Some(Box::new(move |d: Div| d.text_color(rgb(c))));
        }
    }
    if let Some(rest) = class.strip_prefix("border-") {
        if let Some(c) = palette(rest) {
            return Some(Box::new(move |d: Div| d.border_color(rgb(c))));
        }
    }

    None
}

fn parse_spacing(
    class: &str,
    prefix: &str,
    apply: fn(Div, gpui::DefiniteLength) -> Div,
) -> Option<StyleOp> {
    let rest = class.strip_prefix(&format!("{}-", prefix))?;
    let n = rest.parse::<f32>().ok()?;
    Some(Box::new(move |d: Div| apply(d, px(n * 4.0).into())))
}

/// Tailwind palette subset. Returns `0xRRGGBB` for known names.
/// Covers black/white plus slate/gray/blue/red/green/purple/yellow at
/// shades 50, 100, 200, 300, 400, 500, 600, 700, 800, 900.
fn palette(name: &str) -> Option<u32> {
    let basics = [("white", 0xFFFFFF), ("black", 0x000000)];
    for (k, v) in basics {
        if name == k {
            return Some(v);
        }
    }
    let (color, shade) = name.rsplit_once('-')?;
    let shade: u16 = shade.parse().ok()?;
    let table: &[(&str, &[(u16, u32)])] = &[
        (
            "slate",
            &[
                (50, 0xF8FAFC),
                (100, 0xF1F5F9),
                (200, 0xE2E8F0),
                (300, 0xCBD5E1),
                (400, 0x94A3B8),
                (500, 0x64748B),
                (600, 0x475569),
                (700, 0x334155),
                (800, 0x1E293B),
                (900, 0x0F172A),
            ],
        ),
        (
            "gray",
            &[
                (50, 0xF9FAFB),
                (100, 0xF3F4F6),
                (200, 0xE5E7EB),
                (300, 0xD1D5DB),
                (400, 0x9CA3AF),
                (500, 0x6B7280),
                (600, 0x4B5563),
                (700, 0x374151),
                (800, 0x1F2937),
                (900, 0x111827),
            ],
        ),
        (
            "blue",
            &[
                (50, 0xEFF6FF),
                (100, 0xDBEAFE),
                (200, 0xBFDBFE),
                (300, 0x93C5FD),
                (400, 0x60A5FA),
                (500, 0x3B82F6),
                (600, 0x2563EB),
                (700, 0x1D4ED8),
                (800, 0x1E40AF),
                (900, 0x1E3A8A),
            ],
        ),
        (
            "red",
            &[
                (50, 0xFEF2F2),
                (100, 0xFEE2E2),
                (200, 0xFECACA),
                (300, 0xFCA5A5),
                (400, 0xF87171),
                (500, 0xEF4444),
                (600, 0xDC2626),
                (700, 0xB91C1C),
                (800, 0x991B1B),
                (900, 0x7F1D1D),
            ],
        ),
        (
            "green",
            &[
                (50, 0xF0FDF4),
                (100, 0xDCFCE7),
                (200, 0xBBF7D0),
                (300, 0x86EFAC),
                (400, 0x4ADE80),
                (500, 0x22C55E),
                (600, 0x16A34A),
                (700, 0x15803D),
                (800, 0x166534),
                (900, 0x14532D),
            ],
        ),
        (
            "purple",
            &[
                (50, 0xFAF5FF),
                (100, 0xF3E8FF),
                (200, 0xE9D5FF),
                (300, 0xD8B4FE),
                (400, 0xC084FC),
                (500, 0xA855F7),
                (600, 0x9333EA),
                (700, 0x7E22CE),
                (800, 0x6B21A8),
                (900, 0x581C87),
            ],
        ),
        (
            "orange",
            &[
                (50, 0xFFF7ED),
                (100, 0xFFEDD5),
                (200, 0xFED7AA),
                (300, 0xFDBA74),
                (400, 0xFB923C),
                (500, 0xF97316),
                (600, 0xEA580C),
                (700, 0xC2410C),
                (800, 0x9A3412),
                (900, 0x7C2D12),
            ],
        ),
        (
            "yellow",
            &[
                (50, 0xFEFCE8),
                (100, 0xFEF9C3),
                (200, 0xFEF08A),
                (300, 0xFDE047),
                (400, 0xFACC15),
                (500, 0xEAB308),
                (600, 0xCA8A04),
                (700, 0xA16207),
                (800, 0x854D0E),
                (900, 0x713F12),
            ],
        ),
    ];
    for (k, shades) in table {
        if &color == k {
            for (s, v) in *shades {
                if *s == shade {
                    return Some(*v);
                }
            }
        }
    }
    None
}

/// Apply all parsed ops to a fresh `div()` for testing.
#[cfg(test)]
fn apply_all(classes: &[&str]) -> Div {
    let owned: Vec<String> = classes.iter().map(|s| s.to_string()).collect();
    let mut d = gpui::div();
    for op in parse(&owned) {
        d = op(d);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_layout_classes() {
        let _ = apply_all(&["flex", "flex-col", "items-center", "justify-center"]);
    }

    #[test]
    fn parses_spacing_and_color() {
        let _ = apply_all(&["p-4", "gap-2", "bg-slate-900", "text-white"]);
    }

    #[test]
    fn unknown_classes_are_dropped() {
        let ops = parse(&["totally-made-up".into(), "p-2".into()]);
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn parses_typography_and_flex_grow() {
        let _ = apply_all(&["flex-1", "font-bold", "font-semibold", "line-through", "text-5xl"]);
        let ops = parse(&[
            "flex-1".into(),
            "font-bold".into(),
            "text-4xl".into(),
            "text-5xl".into(),
            "line-through".into(),
        ]);
        assert_eq!(ops.len(), 5);
    }

    #[test]
    fn orange_palette_resolves() {
        assert_eq!(palette("orange-500"), Some(0xF97316));
        let ops = parse(&["bg-orange-500".into(), "text-orange-500".into()]);
        assert_eq!(ops.len(), 2);
    }
}
