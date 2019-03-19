use std::mem;

use memmap::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::write_all;
use tokio::prelude::*;

use crate::err::Error;

pub fn write_to_mmap_and_leak<T, E>(
    body: impl Stream<Item = T, Error = E>,
) -> impl Future<Item = &'static [u8], Error = Error>
where
    T: AsRef<[u8]>,
    E: Into<Error>,
{
    future::ok(())
        .and_then(|_| Ok(tempfile()?))
        .and_then(move |file| {
            body.map_err(Into::into)
                .fold(File::from_std(file), |file, chunk| {
                    write_all(file, chunk).map(|(file, _)| file)
                })
        })
        .and_then(|file| {
            // safety: this is an unlinked, exclusive-access temporary file,
            // so it cannot be modified or truncated by anyone else
            let mmap = unsafe { Mmap::map(&file.into_std())? };
            // safety: the mmap will be leaked, and therefore will never be unmapped
            // so the pointed-to data will be valid for the static lifetime
            let data = unsafe { mem::transmute::<&[u8], &'static [u8]>(&*mmap) };
            mem::forget(mmap);
            Ok(data)
        })
}
