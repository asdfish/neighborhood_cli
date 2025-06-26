use {
    dirs::cache_dir,
    std::{
        fmt::{self, Display, Formatter},
        path::PathBuf,
    },
};

pub enum Directory {
    Cache,
}
impl Directory {
    const fn fetcher(&self) -> fn() -> Option<PathBuf> {
        match self {
            Self::Cache => cache_dir,
        }
    }

    pub fn get(self) -> Result<PathBuf, Self> {
        self.fetcher()().ok_or(self).map(|mut path| {
            path.push(crate::NAME);
            path
        })
    }
}
impl Display for Directory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Cache => f.write_str("cache"),
        }
    }
}
