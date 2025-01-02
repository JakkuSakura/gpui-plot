pub fn display_double_smartly(num: f64) -> String {
    let mut formatted = num.to_string();
    while formatted.ends_with("0") {
        formatted.pop();
    }
    if formatted.ends_with(".") {
        formatted.pop();
    }
    formatted
}
