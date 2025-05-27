use std::path::{Path, PathBuf};

pub trait DataLoader {
    fn load<P: AsRef<Path>>(&self, path: P) -> Option<Vec<u8>>;
}

#[cfg(feature = "static_assets")]
mod static_loader {
    use include_dir::{include_dir, Dir};
    use std::path::Path;

    pub struct StaticLoader;

    static ASSETS: Dir = include_dir!("./web/dist/");

    impl super::DataLoader for StaticLoader {
        fn load<P: AsRef<Path>>(&self, path: P) -> Option<Vec<u8>> {
            ASSETS.get_file(path).map(|file| file.contents().to_owned())
        }
    }
}

#[cfg(not(feature = "static_assets"))]
mod dynamic_loader {
    use std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
    };

    pub struct DynamicLoader {
        root: PathBuf,
    }

    impl DynamicLoader {
        pub fn new(root: PathBuf) -> Self {
            Self { root }
        }
    }

    impl super::DataLoader for DynamicLoader {
        fn load<P: AsRef<Path>>(&self, path: P) -> Option<Vec<u8>> {
            let mut full_path = self.root.clone();
            full_path.push(path);
            match File::open(full_path) {
                Ok(file) => Some(file.bytes().map(Result::unwrap).collect()),
                Err(_) => None,
            }
        }
    }
}

pub fn get_loader(_root: PathBuf) -> impl DataLoader {
    #[cfg(feature = "static_assets")]
    {
        tracing::info!("Using static data loader");
        static_loader::StaticLoader
    }
    #[cfg(not(feature = "static_assets"))]
    {
        tracing::info!("Using dynamic data loader");
        dynamic_loader::DynamicLoader::new(_root)
    }
}
