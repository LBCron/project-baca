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

/// Minutes the device clock was east of UTC at `at` (WIB 420), by the
/// browser's own zone rules, so a date past a daylight-saving change still
/// reads right. Outside a browser (native unit tests) there is no zone: 0.
pub fn utc_offset_minutes_at(at: chrono::DateTime<chrono::Utc>) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
            at.timestamp_millis() as f64
        ));
        // getTimezoneOffset counts minutes west of UTC.
        -(date.get_timezone_offset() as i32)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = at;
        0
    }
}

/// The device's offset now, sent with reading activity so a streak day is
/// the reader's own.
pub fn utc_offset_minutes() -> i32 {
    utc_offset_minutes_at(chrono::Utc::now())
}

/// `at` on the reader's wall clock, `offset_minutes` east of UTC.
pub fn local_time(at: chrono::DateTime<chrono::Utc>, offset_minutes: i32) -> chrono::NaiveDateTime {
    (at + chrono::Duration::minutes(i64::from(offset_minutes))).naive_utc()
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
