//! ASCII bar charts for terminal reports.

/// Horizontal bar chart: one labelled bar per row, scaled to `max_width`.
pub fn bar_chart(title: &str, rows: &[(String, f64)], max_width: usize) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push('\n');
    let peak = rows.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    for (label, value) in rows {
        let filled = if peak > 0.0 {
            ((value / peak) * max_width as f64).round() as usize
        } else {
            0
        };
        out.push_str(&format!("{:<12} |{} {:.1}\n", truncate(label, 12), "#".repeat(filled), value));
    }
    out
}

fn truncate(text: &str, width: usize) -> String {
    if text.len() <= width {
        text.to_string()
    } else {
        format!("{}…", &text[..width - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_are_proportional() {
        let chart = bar_chart(
            "times",
            &[("fast".into(), 10.0), ("slow".into(), 20.0)],
            20,
        );
        let fast = chart.lines().nth(1).unwrap();
        let slow = chart.lines().nth(2).unwrap();
        let count = |line: &str| line.matches('#').count();
        assert_eq!(count(fast), 10);
        assert_eq!(count(slow), 20);
    }

    #[test]
    fn empty_rows_do_not_panic() {
        let chart = bar_chart("empty", &[], 10);
        assert_eq!(chart, "empty\n");
    }

    #[test]
    fn labels_are_truncated() {
        let chart = bar_chart(
            "t",
            &[("a-very-long-algorithm-name".into(), 1.0)],
            10,
        );
        assert!(chart.contains('…'));
    }
}
