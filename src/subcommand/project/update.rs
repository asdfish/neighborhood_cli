mod devlog;
mod release;

use {
    crate::MainError,
    clap::ArgMatches,
    futures_lite::stream::{self, StreamExt},
    reqwest::{
        multipart::{Form, Part},
        Client, Response,
    },
    serde::{de::DeserializeOwned, Deserialize},
    std::{
        path::{self, Path, PathBuf},
        pin::{pin, Pin},
        sync::LazyLock,
    },
    tokio::fs,
};

async fn path_to_part<S>(path: S) -> Result<Part, MainError>
where
    S: AsRef<str>,
{
    let file = Part::bytes(
        fs::read(path.as_ref())
            .await
            .map_err(|err| MainError::ReadFile(err, PathBuf::from(path.as_ref())))?,
    )
    .file_name(
        path.as_ref()
            .rsplit_once(path::MAIN_SEPARATOR)
            .map(|(_, r)| r)
            .unwrap_or(path.as_ref())
            .to_string(),
    );
    let file = if let Some(mime) = mime_guess::from_path(path.as_ref()).first() {
        file.mime_str(mime.essence_str())
            .expect("the `mime_guess` crate should be outputting valid mime strings")
    } else {
        file
    };

    Ok(file)
}

pub trait UploadApi: Sized {
    const API: &str;
    type Response: DeserializeOwned;
    type Output: TryFrom<Self::Response, Error = MainError>;

    fn configure(self, _: Form) -> impl Future<Output = Result<Form, MainError>>;
    fn upload(
        self,
        client: &Client,
        token: String,
    ) -> impl Future<Output = Result<Self::Output, MainError>> {
        #[derive(Deserialize)]
        pub struct UploadResponse {
            message: Option<String>,
            url: Option<String>,
        }

        async move {
            client
                .post(Self::API)
                .multipart(self.configure(Form::new().text("token", token)).await?)
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
                .and_then(<Self::Output as TryFrom<Self::Response>>::try_from)
        }
    }
}

#[derive(Deserialize)]
pub struct UploadImagesResponse {
    message: Option<String>,
    urls: Option<Vec<String>>,
}
impl TryFrom<UploadImagesResponse> for Vec<String> {
    type Error = MainError;

    fn try_from(
        UploadImagesResponse { message, urls }: UploadImagesResponse,
    ) -> Result<Vec<String>, MainError> {
        urls.ok_or(message).map_err(MainError::Server)
    }
}
pub struct UploadImages<I>(I)
where
    I: IntoIterator<Item = String>;
impl<I> UploadImages<I>
where
    I: IntoIterator<Item = String>,
{
    pub const fn new(iter: I) -> Self {
        Self(iter)
    }
}
impl<I> UploadApi for UploadImages<I>
where
    I: IntoIterator<Item = String>,
{
    const API: &'static str = "https://express.neighborhood.hackclub.com/upload-images";
    type Response = UploadImagesResponse;
    type Output = Vec<String>;

    async fn configure(self, mut form: Form) -> Result<Form, MainError> {
        let mut files = stream::unfold(
            self.0.into_iter().map(path_to_part),
            |mut iter| async move {
                match iter.next() {
                    Some(part) => Some((part.await, iter)),
                    None => None,
                }
            },
        );

        let mut files = pin!(files);
        while let Some(file) = files.next().await.transpose()? {
            form = form.part("files", file);
        }

        Ok(form)
    }
}

#[derive(Deserialize)]
pub struct UploadVideoResponse {
    message: Option<String>,
    url: Option<String>,
}
impl TryFrom<UploadVideoResponse> for String {
    type Error = MainError;

    fn try_from(
        UploadVideoResponse { message, url }: UploadVideoResponse,
    ) -> Result<String, MainError> {
        url.ok_or(message).map_err(MainError::Server)
    }
}
pub struct UploadVideo<'a>(&'a str);
impl<'a> UploadVideo<'a> {
    pub const fn new(path: &'a str) -> Self {
        Self(path)
    }
}
impl UploadApi for UploadVideo<'_> {
    const API: &'static str = "https://express.neighborhood.hackclub.com/upload-video";
    type Response = UploadVideoResponse;
    type Output = String;

    async fn configure(self, form: Form) -> Result<Form, MainError> {
        Ok(form.part("file", path_to_part(self.0).await?))
    }
}

pub fn execute(mut args: ArgMatches, name: &str) -> Result<(), MainError> {
    let async_upload = args.remove_one("async").unwrap_or_default();
    let message = args.remove_one::<String>("message").unwrap();
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "devlog" => devlog::execute(args, name, async_upload, message.as_str()),
        "release" | "ship" => release::execute(args, name, async_upload, message),
        _ => unreachable!(),
    }
}
