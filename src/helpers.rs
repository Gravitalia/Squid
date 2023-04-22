use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use lingua::Language::{English, French, Spanish};
use anyhow::Result;
use std::fs;

/// Copy a file from a folder to another one
pub fn copy_file(src: &str, dst: &str) -> Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

/// Copy a directory to another one
pub fn copy_dir(src: &str, dst: &str) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let dst_path = format!("{}/{}", dst, file_name);

        if file_name == "lang" {
            copy_dir(path.to_str().unwrap(), &format!("{}/lang", dst))?;
        } else {
            copy_file(path.to_str().unwrap(), &dst_path)?;
        }
    }

    Ok(())
}

/// Detect a language between english, french and spanish
pub fn detect_language(text: String) -> Result<Option<&'static str>> {
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&vec![English, French, Spanish]).build();
    let detected_language: Option<Language> = detector.detect_language_of(text);

    if detected_language.is_none() {
        return Ok(None);
    }

    match detected_language.unwrap() {
        English => Ok(Some("en")),
        French => Ok(Some("fr")),
        Spanish => Ok(Some("es"))
    }
}