pub mod grf;
pub mod grf_event;

/// format a vec of strings as a multi-line TOML array
pub fn format_var_array(vars: &[String]) -> String {
    if vars.is_empty() {
        return "[]".to_string();
    }
    let items: Vec<String> = vars.iter().map(|v| format!("\"{}\"", v)).collect();
    format!("[\n  {}\n]", items.join(",\n  "))
}

/// format a vec of strings as a single-line TOML array
pub fn format_string_array(vars: &[String]) -> String {
    if vars.is_empty() {
        return "[]".to_string();
    }
    let items: Vec<String> = vars.iter().map(|v| format!("\"{}\"", v)).collect();
    format!("[{}]", items.join(", "))
}
