// Port of numeric-input.js: accepts one decimal separator (`.` or `,`) and rejects anything else.

/// Returns `None` for malformed text, non-finite numbers, or values below `min`.
pub fn parse_numeric_input(text: &str, min: f64) -> Option<f64> {
    let trimmed = text.trim();
    let unsigned = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
    let (int_part, frac_part) = match unsigned.split_once(['.', ',']) {
        Some((i, f)) => (i, Some(f)),
        None => (unsigned, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let valid = match frac_part {
        None => digits(int_part),
        Some(f) => (int_part.is_empty() || digits(int_part)) && digits(f),
    };
    if !valid {
        return None;
    }
    let number: f64 = trimmed.replacen(',', ".", 1).parse().ok()?;
    (number.is_finite() && number >= min).then_some(number)
}

#[cfg(test)]
mod tests {
    use super::parse_numeric_input as p;

    #[test]
    fn accepts_comma_and_dot_decimals() {
        for text in ["50,0", "50.0", " 50,0 "] {
            assert_eq!(p(text, 0.0), Some(50.0), "{text}");
        }
        assert_eq!(p("0,5", 0.0), Some(0.5));
        assert_eq!(p(".25", 0.0), Some(0.25));
        assert_eq!(p("-12,5", f64::NEG_INFINITY), Some(-12.5));
    }

    #[test]
    fn rejects_incomplete_ambiguous_nonfinite_and_below_minimum() {
        for text in ["", "50,", "1,2,3", "1.000,5", "Infinity", "1e309", "abc"] {
            assert_eq!(p(text, 0.0), None, "{text}");
        }
        assert_eq!(p("-1", 0.0), None);
    }
}
