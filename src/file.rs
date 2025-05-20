use chardetng::EncodingDetector;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

/// Wrapper around File for better encoding handling. Rust Strings only supports
/// valid UTF-8 encodings. This should work for the most part. Latin-1 encoding
/// is buggy.
pub struct OrgFile {
    file: File,
}

impl OrgFile {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(Self {
            file: File::open(path)?,
        })
    }

    pub fn read_to_string(&mut self) -> io::Result<String> {
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;

        let mut detector = EncodingDetector::new();
        detector.feed(&buffer, true);
        let encoding = detector.guess(None, true);

        if encoding.output_encoding() != encoding_rs::UTF_8 {
            tracing::warn!(
                "Reading non UTF-8 ({}) file {:?}",
                encoding.name(),
                self.file
            );
        }

        let (cow, _, transformations) = encoding.decode(&buffer);

        if transformations {
            tracing::info!("There were malformed sequences in {:?}", self.file);
        }

        Ok(cow.into_owned())
    }
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = OrgFile::open(path)?;
    file.read_to_string()
}
