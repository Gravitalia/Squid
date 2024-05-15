use std::{fs::OpenOptions, io::Write};

#[tokio::main]
async fn main() {
    // Download dataset.
    let resp = reqwest::get(
        "https://drive.usercontent.google.com/download?id=1aOYNDj8Rj6iI-nWJkXkAF43bNtoR7LXh&export=download&authuser=0&confirm=t&uuid=c2215629-8193-4f40-9737-1ef1ed1b96e8&at=APZUnTU595qAIh9-PjIMCEUs7obD:1709837225992",
    )
    .await
    .unwrap()
    .bytes()
    .await
    .unwrap();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("./wikisent2.txt")
        .unwrap();

    file.write_all(resp.as_ref()).unwrap();

    // Decompress it.
    /*let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {i} comment: {comment}");
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .unwrap();
            }
        }
    }

    fs::remove_file("./archive.zip").unwrap();*/
}
