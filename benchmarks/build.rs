use std::fs;
use std::{fs::OpenOptions, io, io::Write};

#[tokio::main]
async fn main() {
    // Download dataset.
    let resp = reqwest::get(
        "https://storage.googleapis.com/kaggle-data-sets/46601/84740/bundle/archive.zip?X-Goog-Algorithm=GOOG4-RSA-SHA256&X-Goog-Credential=gcp-kaggle-com%40kaggle-161607.iam.gserviceaccount.com%2F20240222%2Fauto%2Fstorage%2Fgoog4_request&X-Goog-Date=20240222T221154Z&X-Goog-Expires=259200&X-Goog-SignedHeaders=host&X-Goog-Signature=66e60b5ca37b719240daf1622e6267f5bd26783af2cbaea38e8f4d3729e3150f3f8abbbe68503f2e6adfb97fd6865b41e5b630611c9286a5dad928c1ed2b4b6dd7fd0593d11a33bc42f1a28b257fd4598a0236b1855527f59e469648994d85769ba65381f4cc9af71dfc43c97779c0fb958236810194d2626da281a97a05a6bcf25ae1a3cf123ae3d250324ac4d89a652470080b27c55a2814844e87f81d6b5dc7780cc10a25b161de2f6fddc033db284aaf1fb4cd70e3d6c3b49bebd178ea902bc4bee69409ed77c5eeca5ca931f2121df21ebf46113bd4f609f6fe66b35be243e1fb137dbd6f0bb2f80b263d06f068d707a2009419e8bf30355ff88ceeb96d",
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
        .open("./archive.zip")
        .unwrap();

    file.write(resp.as_ref()).unwrap();

    // Decompress it.
    let mut archive = zip::ZipArchive::new(file).unwrap();

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

    fs::remove_file("./archive.zip").unwrap();
}
