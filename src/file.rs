use hyper::body::HttpBody;
use hyper::Body;
use memmap2::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::err::Error;

pub async fn write_to_mmap(mut body: Body) -> Result<Mmap, Error> {
    let file = tempfile()?;

    let mut file = File::from_std(file);
    while let Some(bytes) = body.data().await {
        let bytes = bytes?;
        file.write_all(&bytes).await?;
    }
    let file = file.into_std().await;

    // safety: this is an unlinked, exclusive-access temporary file,
    // so it cannot be modified or truncated by anyone else
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(mmap)
}
