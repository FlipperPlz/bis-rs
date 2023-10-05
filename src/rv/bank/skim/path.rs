const WIN_DIR: char = '\\';
const UNIX_DIR: char = '/';

#[inline]
pub fn normalize_path(path: &str, directory: bool) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    let mut result = Vec::with_capacity(path.len());
    let mut last_was_separator = true;

    for c in path.chars() {
        match c {
            UNIX_DIR | WIN_DIR => {
                if last_was_separator { continue }

                result.push(WIN_DIR);
                last_was_separator = true;
            },
            _ => {
                last_was_separator = false;
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    if !directory && !result.is_empty() && *result.last().unwrap() == WIN_DIR {
        result.pop();
    }

    result.iter().collect()
}

#[inline]
pub fn convert_dir_slash(name: &String) -> String {
    if !name.contains(UNIX_DIR) {
        return name.clone();
    }

    name.replace(UNIX_DIR, "\\")
}