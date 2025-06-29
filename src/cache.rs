use {
    crate::MainError,
    cfg_if::cfg_if,
    std::{
        borrow::Cow,
        fmt::{self, Display, Formatter, Write},
        fs,
        path::{Path, PathBuf},
        sync::LazyLock,
    },
    zeroize::{Zeroize, Zeroizing},
};

pub static ROOT: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    dirs::cache_dir().map(|mut root| {
        root.push(crate::NAME);
        root
    })
});
pub static PROJECT_TOKENS: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    ROOT.as_ref().map(PathBuf::from).clone().map(|mut root| {
        root.push("project_tokens");
        root
    })
});
pub static TOKEN: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    ROOT.as_ref().map(PathBuf::from).map(|mut root| {
        root.push("token");
        root
    })
});
pub static RELEASE: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    ROOT.as_ref().map(PathBuf::from).map(|mut root| {
        root.push("release");
        root
    })
});

pub fn read_token() -> Result<String, MainError> {
    TOKEN.as_ref().ok_or(MainError::GetCache).and_then(|path| {
        fs::read_to_string(&path).map_err(|error| MainError::ReadFile(error, Cow::Borrowed(path)))
    })
}

// #[derive(Default)]
// pub struct PathCache {
//     root: Option<PathBuf>,
//     project_tokens: Option<PathBuf>,
//     token: Option<PathBuf>,
//     release: Option<PathBuf>,
// }
// impl PathCache {

//     // fn set_root_raw(root: &mut Option<PathBuf>) -> Option<&Path> {
//     //     if root.is_none() {
//     //         *root = dirs::cache_dir().map(|mut root| {
//     //             root.push(crate::NAME);
//     //             root
//     //         });
//     //     }

//     //     root.as_deref()
//     // }
//     // pub fn set_root(&mut self) -> Result<&Path, GetCacheError> {
//     //     Self::set_root_raw(&mut self.root).ok_or(GetCacheError)
//     // }

//     // fn set_branch<'a>(
//     //     root: &mut Option<PathBuf>,
//     //     branch: &'a mut Option<PathBuf>,
//     //     file_name: &str,
//     // ) -> Result<&'a Path, GetCacheError> {
//     //     if branch.is_none() {
//     //         *branch = Self::set_root_raw(root)
//     //             .as_deref()
//     //             .map(PathBuf::from)
//     //             .map(|mut path| {
//     //                 path.push(file_name);
//     //                 path
//     //             });
//     //     }

//     //     branch.as_deref().ok_or(GetCacheError)
//     // }
//     // pub fn set_token(&mut self) -> Result<&Path, GetCacheError> {
//     //     Self::set_branch(&mut self.root, &mut self.token, "token")
//     // }
//     // pub fn set_project_tokens(&mut self) -> Result<&Path, GetCacheError> {
//     //     Self::set_branch(&mut self.root, &mut self.project_tokens, "project_tokens")
//     // }
//     // pub fn set_release(&mut self) -> Result<&Path, GetCacheError> {
//     //     Self::set_branch(&mut self.root, &mut self.release, "release")
//     // }

//     // pub fn get_token(&self) -> Result<&Path, GetCacheError> {
//     //     self.token.as_deref().ok_or(GetCacheError)
//     // }
//     // pub fn get_project_tokens(&mut self) -> Result<&Path, GetCacheError> {
//     //     self.project_tokens.as_deref().ok_or(GetCacheError)
//     // }
//     // pub fn get_release(&self) -> Result<&Path, GetCacheError> {
//     //     self.release.as_deref().ok_or(GetCacheError)
//     // }

//     // pub fn into_root(mut self) -> Result<PathBuf, GetCacheError> {
//     //     self.root
//     //         .take()
//     //         .or_else(|| {
//     //             Self::set_root_raw(&mut self.root);
//     //             self.root.take()
//     //         })
//     //         .ok_or(GetCacheError)
//     // }
//     // fn into_branch(
//     //     root: &mut Option<PathBuf>,
//     //     branch: &mut Option<PathBuf>,
//     //     file_name: &str,
//     // ) -> Result<PathBuf, GetCacheError> {
//     //     branch
//     //         .take()
//     //         .or_else(|| {
//     //             Self::set_root_raw(root);
//     //             root.take().map(|mut root| {
//     //                 root.push(file_name);
//     //                 root
//     //             })
//     //         })
//     //         .ok_or(GetCacheError)
//     // }
//     // pub fn into_token(mut self) -> Result<PathBuf, GetCacheError> {
//     //     Self::into_branch(&mut self.root, &mut self.token, "token")
//     // }
//     // pub fn into_release(mut self) -> Result<PathBuf, GetCacheError> {
//     //     Self::into_branch(&mut self.root, &mut self.release, "release")
//     // }

//     // pub fn read_project_token(&mut self, project: &str) -> Result<String, MainError> {
//     //     let mut project_path = self.set_project_tokens()?.to_path_buf();
//     //     project_path.push(project);

//     //     if project_path.is_file() {
//     //         fs::read_to_string(&project_path)
//     //             .map_err(|error| MainError::ReadFile(error, project_path))
//     //     } else {
//     //         todo!()
//     //     }
//     // }

//     // pub fn read_token(&mut self) -> Result<String, MainError> {
//     //     self.set_token()
//     //         .map_err(MainError::GetCache)
//     //         .and_then(|path| {
//     //             fs::read_to_string(&path)
//     //                 .map_err(|error| MainError::ReadFile(error, path.to_path_buf()))
//     //         })
//     // }
//     // pub fn try_read_token(&self) -> Result<String, MainError> {
//     //     self.get_token()
//     //         .map_err(MainError::GetCache)
//     //         .and_then(|path| {
//     //             fs::read_to_string(path)
//     //                 .map_err(|error| MainError::ReadFile(error, path.to_path_buf()))
//     //         })
//     // }
// }
