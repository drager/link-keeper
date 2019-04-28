use crate::file_handling::FileHandling;
use crate::Link;
use serde_json;
use std::path::PathBuf;

pub fn add_to_raw(path: PathBuf, file_name: String, new_link: Link) -> Result<(), failure::Error> {
    let file_handling = FileHandling::new(path, file_name);
    file_handling.create_file()?;

    let formatted_data = format_data(&vec![new_link])?;

    let formatted_data = if file_handling.file_is_empty()? {
        formatted_data
    } else {
        let old_contents = file_handling.read_data_from_file()?;

        let mut old_contents_as_orginal = to_orginal_format(&old_contents)?;

        old_contents_as_orginal.append(&mut to_orginal_format(&formatted_data)?);

        format_data(&old_contents_as_orginal)?
    };

    file_handling.write_to_file(formatted_data.as_bytes())?;

    Ok(())
}

fn format_data(links: &Vec<Link>) -> Result<String, serde_json::error::Error> {
    serde_json::to_string(links)
}

fn to_orginal_format(contents: &str) -> Result<Vec<Link>, serde_json::error::Error> {
    serde_json::from_str::<_>(contents)
}
