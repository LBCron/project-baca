//! Small presentation helpers shared by pages.

const WORDS_PER_MINUTE: i32 = 230;

/// Chapter numerals in the style of a printed table of contents.
pub fn roman(n: i32) -> String {
    if n <= 0 || n >= 4000 {
        return n.to_string();
    }
    const TABLE: [(i32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut rest = n;
    let mut out = String::new();
    for (value, glyph) in TABLE {
        while rest >= value {
            out.push_str(glyph);
            rest -= value;
        }
    }
    out
}

pub fn reading_minutes(words: i32) -> i32 {
    (words / WORDS_PER_MINUTE).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roman_numerals_match_print_convention() {
        assert_eq!(roman(1), "I");
        assert_eq!(roman(4), "IV");
        assert_eq!(roman(9), "IX");
        assert_eq!(roman(14), "XIV");
        assert_eq!(roman(40), "XL");
        assert_eq!(roman(0), "0");
    }

    #[test]
    fn reading_minutes_never_zero() {
        assert_eq!(reading_minutes(0), 1);
        assert_eq!(reading_minutes(2300), 10);
    }
}
