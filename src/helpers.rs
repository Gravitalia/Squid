use std::fs;
use std::io;

// Copy a file from a folder to another one
pub fn copy_file(src: &str, dst: &str) -> io::Result<()> {
    fs::copy(src, dst)?;
    Ok(())
}

// Copy a directory to another one
pub fn copy_dir(src: &str, dst: &str) -> io::Result<()> {
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
