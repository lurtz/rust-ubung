use std::io::{Result, Write};

use ffi::create_int_wrapper;

#[cxx::bridge(namespace = "org::blobstore")]
mod ffi {
    // Shared structs with fields visible to both languages.
    struct BlobMetadata {
        size: usize,
        tags: Vec<String>,
    }

    // Rust types and signatures exposed to C++.
    extern "Rust" {
        type MultiBuf;

        fn next_chunk(buf: &mut MultiBuf) -> &[u8];
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("cxx-experiment/include/blobstore.h");

        type BlobstoreClient;

        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;
        fn put(self: Pin<&mut BlobstoreClient>, parts: &mut MultiBuf) -> u64;
        fn tag(self: Pin<&mut BlobstoreClient>, blobid: u64, tag: &str);
        fn metadata(self: Pin<&mut BlobstoreClient>, blobid: u64) -> BlobMetadata;

        type Int_wrapper;
        fn two_times(self: &Int_wrapper) -> u8;
        fn create_int_wrapper(val: u8) -> UniquePtr<Int_wrapper>;

    }
}

// An iterator over contiguous chunks of a discontiguous file object.
//
// Toy implementation uses a Vec<Vec<u8>> but in reality this might be iterating
// over some more complex Rust data structure like a rope, or maybe loading
// chunks lazily from somewhere.
pub struct MultiBuf {
    chunks: Vec<Vec<u8>>,
    pos: usize,
}
pub fn next_chunk(buf: &mut MultiBuf) -> &[u8] {
    let next = buf.chunks.get(buf.pos);
    buf.pos += 1;
    next.map_or(&[], Vec::as_slice)
}

fn main_impl(logger: &mut dyn Write) -> Result<()> {
    let mut client = ffi::new_blobstore_client();

    // Upload a blob.
    let chunks = vec![b"fearless".to_vec(), b"concurrency".to_vec()];
    let mut buf = MultiBuf { chunks, pos: 0 };
    let blobid = client.pin_mut().put(&mut buf);
    writeln!(logger, "blobid = {blobid}")?;

    // Add a tag.
    client.pin_mut().tag(blobid, "rust");

    // Read back the tags.
    let metadata = client.pin_mut().metadata(blobid);
    writeln!(logger, "tags = {:?}", metadata.tags)?;

    let int_wrapper = create_int_wrapper(23);
    writeln!(logger, "double == {}", int_wrapper.two_times())?;
    Ok(())
}

#[cfg(not(test))]
fn main() -> Result<()> {
    main_impl(&mut std::io::stdout())
}

#[cfg(test)]
mod test {
    use crate::main_impl;

    #[test]
    fn main_works() {
        let mut logger = Vec::new();
        main_impl(&mut logger).unwrap();
        let expected = "blobid = 9851996977040795552\ntags = [\"rust\"]\ndouble == 46\n".as_bytes();
        assert_eq!(expected, &logger);
    }
}
