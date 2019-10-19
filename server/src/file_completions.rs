use crate::errors::Try;
use anyhow::anyhow;
use std::path::Path;

pub fn complete_file_path(prefix: &str) -> Try<Vec<String>> {
    let index_of_last_slash = prefix
        .rfind('/')
        .or_else(|| prefix.rfind('\\'))
        .unwrap_or(prefix.len() - 1);
    let (directory, prefix) = prefix.split_at(index_of_last_slash + 1);
    let mut result = Vec::new();
    for file in Path::new(directory).read_dir()? {
        let name = file?
            .file_name()
            .into_string()
            .map_err(|s| anyhow!("invalid file name {:?}", s))?;
        if name.starts_with(prefix) {
            result.push([directory, &name].concat())
        }
    }
    Ok(result)
}
