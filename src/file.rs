use failure::Error;
use futures::{future, Future, Stream};
use memmap::Mmap;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::write_all;

pub fn write_to_mmap<T, E>(
    body: impl Stream<Item = T, Error = E>,
) -> impl Future<Item = Mmap, Error = Error>
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
            unsafe { Ok(Mmap::map(&file.into_std())?) }
        })
}
