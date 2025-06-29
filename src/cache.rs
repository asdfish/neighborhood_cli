use {
    crate::MainError,
    cfg_if::cfg_if,
    reqwest::{multipart::Form, Client, Response},
    serde::Deserialize,
    std::{
        borrow::Cow,
        fmt::{self, Display, Formatter, Write},
        fs::{self, DirBuilder},
        path::{Path, PathBuf},
        sync::LazyLock,
    },
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

pub fn create_if_not_dir(path: Cow<'static, Path>) -> Result<(), MainError> {
    if path.is_dir() {
        return Ok(());
    }

    DirBuilder::new()
        .recursive(true)
        .create(&path)
        .map_err(|error| MainError::CreateDirectory(error, path))
}

#[derive(Deserialize)]
struct GetUserAppsResponse {
    message: Option<String>,
    apps: Option<Vec<App>>,
}
#[derive(Deserialize)]
pub struct App {
    id: String,
    name: String,
}

pub async fn get_project_token(project: Cow<'_, str>) -> Result<String, MainError> {
    let project_token = PROJECT_TOKENS.as_ref().ok_or(MainError::GetCache)?;

    create_if_not_dir(Cow::Borrowed(project_token))?;

    let mut project_token_path = project_token.to_path_buf();
    project_token_path.push(project.as_ref());

    if project_token_path.is_file() {
        fs::read_to_string(&project_token_path)
            .map_err(|error| MainError::ReadFile(error, Cow::Owned(project_token_path)))
    } else {
        let token = read_token()?;

        use serde::Serialize;
        #[derive(Serialize)]
        struct Body {
            token: String,
        }

        Client::builder()
            .build()
            .map_err(MainError::CreateClient)?
            .get(&format!(
                "https://neighborhood.hackclub.com/api/getUserApps?token={token}"
            ))
            .send()
            .await
            .and_then(Response::error_for_status)
            .map_err(MainError::ExecuteRequest)?
            .text()
            .await
            .map_err(MainError::ExecuteRequest)
            .and_then(|response| {
                serde_json::from_str(&response)
                    .map_err(|error| MainError::DecodeResponse(error, response))
            })
            .and_then(|GetUserAppsResponse { apps, message }| {
                apps.ok_or(MainError::Server(message))
            })
            .and_then(|apps| {
                apps.into_iter()
                    .fold(None, |accum, App { id, name }| {
                        let mut path = project_token.clone();
                        path.push(&name);
                        let _ = fs::write(path, &id);

                        if name == project {
                            Some(id)
                        } else {
                            accum
                        }
                    })
                    .ok_or_else(|| MainError::NonExistantProject(project.into_owned()))
            })
    }
}
pub fn read_token() -> Result<String, MainError> {
    TOKEN
        .as_ref()
        .ok_or(MainError::GetCache)
        .and_then(|path| fs::read_to_string(&path).map_err(|_| MainError::GetToken))
}
