use anyhow::Result;
use fancy_regex::Regex;
use std::fs::File;
use std::io::Read;

fn main() {
    
    match verify_package() {
        Ok(ok) => {
            println!("File passed verification - {}", ok);
        }
        Err(e) => {
            println!("{:#?}", e)
        }
    };
}

fn verify_package() -> Result<bool> {
    let filename = "src/package.toml";
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let name_re = Regex::new(r#"name\s?=\s?"[^"\s]*""#).unwrap();
    let description_re = Regex::new(r#"description\s?=\s?"[[:alnum:]\s]*""#).unwrap();
    let version_re = Regex::new(r#"version\s?=\s?\"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?""#).unwrap();
    let content_type_re = Regex::new(r#"type\s=\s"vm""#).unwrap();
    let content_sub_type_re = Regex::new(r#"sub_type\s=\s"packer""#).unwrap();

    let name_result = name_re.is_match(&contents)?;
    let description_result = description_re.is_match(&contents)?;
    let version_result = version_re.is_match(&contents)?;
    let content_type_result = content_type_re.is_match(&contents)?;
    let content_sub_type_result = content_sub_type_re.is_match(&contents)?;

    println!("Name passed - {}", name_result);
    println!("Description passed - {}", description_result);
    println!("Version passed - {}", version_result);
    println!("Content type passed - {}", content_type_result);
    println!("Content sub-type passed - {}", content_sub_type_result);

    return if !name_result
        || !description_result
        || !version_result
        || !content_type_result
        || !content_sub_type_result
    {
        Ok(false)
    } else {
        Ok(true)
    };
}

#[cfg(test)]
mod tests {
    use std::fs::remove_file;
    use std::io::Write;
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn positive_result_on_all_correct_fields() {
        let path = std::path::Path::new("src/package.toml");
        let mut file = File::create(path).unwrap();
        file.write_all(
            br#"
[package]
name = "test_package1-1"
description = "this package does nothing at all"
version = "1.0.0"
authors = ["Robert robert@exmaple.com"]
[content]
type = "vm"
sub_type = "packer"
"#,
        )
        .unwrap();

        assert_eq!(verify_package().unwrap(), true);

        match remove_file(path) {
            Ok(_) => {}
            Err(e) => panic!("Could not delete file, {}", e),
        }
    }

    #[test]
    #[serial]
    fn negative_result_on_all_incorrect_fields() {
        let path = std::path::Path::new("src/package.toml");
        let mut file = File::create(path).unwrap();
        file.write_all(
            br#"
[package]
name = "test package 3"
description = "This packaged appeared @eventcon #package_power"
version = "v5"
authors = ["Robert robert@exmaple.com"]
[content]
type = "virtuelle machine"
sub_type = ""
"#,
        )
        .unwrap();

        assert_eq!(verify_package().unwrap(), false);
        match remove_file(path) {
            Ok(_) => {}
            Err(e) => panic!("Could not delete file, {}", e),
        }
    }
}
