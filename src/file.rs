use std::mem::{self, ManuallyDrop};

use memmap::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::prelude::*;
use tokio::stream::{Stream, StreamExt};

use crate::err::Error;

pub async fn write_to_mmap_and_leak<T, E>(
    mut body: impl Stream<Item = Result<T, E>> + Unpin,
) -> Result<&'static [u8], Error>
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
    let mmap = ManuallyDrop::new(unsafe { Mmap::map(&file)? });
    // safety: the mmap will be leaked, and therefore can never be unmapped
    // so the pointed-to data will be valid for the static lifetime
    let data = unsafe { mem::transmute::<&[u8], &'static [u8]>(&*mmap) };

    Ok(data)
}
