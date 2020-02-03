use memmap::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::prelude::*;
use tokio::stream::{Stream, StreamExt};

use crate::err::Error;

pub async fn write_to_mmap<T, E>(
    mut body: impl Stream<Item = Result<T, E>> + Unpin,
) -> Result<Mmap, Error>
where
    T: AsRef<[u8]>,
    E: Into<Error>,
{
    let file = tempfile()?;

    let mut file = File::from_std(file);
    while let Some(bytes) = body.next().await {
        let bytes = bytes.map_err(Into::into)?;
        file.write_all(bytes.as_ref()).await?;
    }
    let file = file.into_std().await;

    // safety: this is an unlinked, exclusive-access temporary file,
    // so it cannot be modified or truncated by anyone else
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(mmap)
}
