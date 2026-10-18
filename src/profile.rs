use anyhow::{Context, Result};
use std::{fs::File, io::Read, path::Path};

/// Read a color profile file and strip the JSON extensions we allow
/// (// comments, trailing comma after the last element), so the result
/// can be passed to `cherryrgb::read_color_profile`.
pub fn read_profile_json(path: &Path) -> Result<String> {
    let mut f = File::open(path).context(format!("color profile {:?}", path))?;
    let mut json: String = String::new();

    f.read_to_string(&mut json)?;
    // Allow // comments
    let re = regex::RegexBuilder::new(r"//.*?$")
        .multi_line(true)
        .build()
        .unwrap();
    json = re.replace_all(&json, "").to_string();
    // Allow trailing comma after last element
    let re = regex::RegexBuilder::new(r",(\s*\})").build().unwrap();
    json = re.replace_all(&json, "$1").to_string();

    log::debug!("{json}");

    Ok(json)
}
