use std::path::PathBuf;

use anyhow::Result;

pub fn generate_package_path(package_name: String) -> Result<PathBuf> {
    let mut path = PathBuf::new();
    match package_name.len() {
        1 => path.push("1"),
        2 => path.push("2"),
        3 => {
            path.push("3");
            path.push(package_name[0..1].to_string());
        }
        _ => {
            path.push(package_name[0..2].to_string());
            path.push(package_name[2..4].to_string());
        }
    };
    path.push(package_name);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::generate_package_path;
    use anyhow::Result;

    #[test]
    fn correct_package_path_for_single_character_names() -> Result<()> {
        let expected_path: PathBuf = ["1", "t"].iter().collect();
        let path = generate_package_path("t".to_string())?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_two_character_names() -> Result<()> {
        let expected_path: PathBuf = ["2", "te"].iter().collect();
        let path = generate_package_path("te".to_string())?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_three_charcter_names() -> Result<()> {
        let expected_path: PathBuf = ["3", "t", "tes"].iter().collect();
        let path = generate_package_path("tes".to_string())?;

        assert_eq!(expected_path, path);
        Ok(())
    }

    #[test]
    fn correct_package_path_for_four_or_more_charcter_names() -> Result<()> {
        let expected_path: PathBuf = ["my", "-t", "my-test"].iter().collect();
        let path = generate_package_path("my-test".to_string())?;

        assert_eq!(expected_path, path);
        Ok(())
    }
}
