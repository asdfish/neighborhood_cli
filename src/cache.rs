use {
    crate::MainError,
    cfg_if::cfg_if,
    std::{
        fmt::{self, Display, Formatter, Write},
        fs,
        path::{Path, PathBuf},
    },
};

#[derive(Default)]
pub struct PathCache {
    root: Option<PathBuf>,
    token: Option<PathBuf>,
    ship: Option<PathBuf>,
}
impl PathCache {
    fn set_root_raw(root: &mut Option<PathBuf>) -> Option<&Path> {
        if root.is_none() {
            *root = dirs::cache_dir().map(|mut root| {
                root.push(crate::NAME);
                root
            });
        }

        root.as_deref()
    }
    pub fn set_root(&mut self) -> Result<&Path, GetCacheError> {
        Self::set_root_raw(&mut self.root).ok_or(GetCacheError)
    }

    fn set_branch<'a>(
        root: &mut Option<PathBuf>,
        branch: &'a mut Option<PathBuf>,
        file_name: &str,
    ) -> Result<&'a Path, GetCacheError> {
        if branch.is_none() {
            *branch = Self::set_root_raw(root)
                .as_deref()
                .map(PathBuf::from)
                .map(|mut path| {
                    path.push(file_name);
                    path
                });
        }

        branch.as_deref().ok_or(GetCacheError)
    }
    pub fn set_token(&mut self) -> Result<&Path, GetCacheError> {
        Self::set_branch(&mut self.root, &mut self.token, "token")
    }
    pub fn set_ship(&mut self) -> Result<&Path, GetCacheError> {
        Self::set_branch(&mut self.root, &mut self.ship, "ship")
    }

    pub fn get_token(&self) -> Result<&Path, GetCacheError> {
        self.token.as_deref().ok_or(GetCacheError)
    }
    pub fn get_ship(&self) -> Result<&Path, GetCacheError> {
        self.ship.as_deref().ok_or(GetCacheError)
    }

    pub fn into_root(mut self) -> Result<PathBuf, GetCacheError> {
        self.root
            .take()
            .or_else(|| {
                Self::set_root_raw(&mut self.root);
                self.root.take()
            })
            .ok_or(GetCacheError)
    }
    fn into_branch(
        root: &mut Option<PathBuf>,
        branch: &mut Option<PathBuf>,
        file_name: &str,
    ) -> Result<PathBuf, GetCacheError> {
        branch
            .take()
            .or_else(|| {
                Self::set_root_raw(root);
                root.take().map(|mut root| {
                    root.push(file_name);
                    root
                })
            })
            .ok_or(GetCacheError)
    }
    pub fn into_token(mut self) -> Result<PathBuf, GetCacheError> {
        Self::into_branch(&mut self.root, &mut self.token, "token")
    }
    pub fn into_ship(mut self) -> Result<PathBuf, GetCacheError> {
        Self::into_branch(&mut self.root, &mut self.ship, "ship")
    }

    pub fn read_token(&mut self) -> Result<String, MainError> {
        self.set_token()
            .map_err(MainError::GetCache)
            .and_then(|path| fs::read_to_string(path).map_err(|_| MainError::GetToken))
    }
    pub fn try_read_token(&self) -> Result<String, MainError> {
        self.get_token()
            .map_err(MainError::GetCache)
            .and_then(|path| fs::read_to_string(path).map_err(|_| MainError::GetToken))
    }
}

#[derive(Debug)]
pub struct GetCacheError;
impl Display for GetCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("failed to get the cache directory, please ensure that you have the following environment variables set:")
                .and_then(|_| {
                    cfg_if! {
                        if #[cfg(target_os = "macos")] {
                            const ENV_VARS: &[&str] = &["HOME"];
                        } else if #[cfg(unix)] {
                            const ENV_VARS: &[&str] = &["XDG_CACHE_HOME", "HOME"];
                        } else if #[cfg(windows)] {
                            const ENV_VARS: &[&str] = &["LOCALAPPDATA"];
                        } else {
                            const ENV_VARS: &[&str] = &[];
                        }
                    }

                    ENV_VARS
                        .iter()
                        .try_for_each(|env_var| {
                            f
                                .write_str(env_var)
                                .and_then(|_| f.write_char('\n'))
                        })
                })
    }
}
