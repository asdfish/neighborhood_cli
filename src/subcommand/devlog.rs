use {
    crate::{api::MessageResponse, cache::PathCache, MainError},
    clap::ArgMatches,
    futures_lite::future,
    reqwest::{
        multipart::{Form, Part},
        Client, Response,
    },
    serde::{Deserialize, Serialize},
    std::{
        future::Future,
        path::{self, PathBuf},
    },
    tokio::{fs, runtime},
};

pub trait FuturesExt: Future {
    fn inspect<F>(self, mut inspection: F) -> impl Future<Output = Self::Output>
    where
        Self: Sized,
        F: FnMut(&Self::Output),
    {
        async move {
            let output = self.await;
            inspection(&output);
            output
        }
    }
}

async fn upload(client: &Client, token: String, path: &str) -> Result<String, MainError> {
    #[derive(Deserialize)]
    pub struct UploadVideoResponse {
        message: Option<String>,
        url: Option<String>,
    }

    let file = Part::bytes(
        fs::read(path)
            .await
            .map_err(|err| MainError::ReadFile(err, PathBuf::from(path)))?,
    )
    .file_name(
        path.rsplit_once(path::MAIN_SEPARATOR)
            .map(|(_, r)| r)
            .unwrap_or(path)
            .to_string(),
    );
    let file = if let Some(mime) = mime_guess::from_path(path).first() {
        file.mime_str(mime.essence_str())
            .expect("the `mimi_guess` crate should be outputting valid mime strings")
    } else {
        file
    };

    eprintln!("Uploading {path}");
    let UploadVideoResponse { url, message } = client
        .post("https://express.neighborhood.hackclub.com/upload-video")
        .multipart(Form::new().text("token", token).part("file", file))
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
        })?;
    eprintln!("Uploaded {path}");

    url.ok_or(MainError::Server(message))
}

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let app = args.remove_one::<String>("app").unwrap();
    let photobooth = args.remove_one::<String>("photobooth").unwrap();
    let demo = args.remove_one::<String>("demo").unwrap();
    let message = args.remove_one::<String>("message").unwrap();

    let token = PathCache::default().read_token()?;

    let runtime = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(MainError::CreateRuntime)?;

    let client = Client::builder().build().map_err(MainError::CreateClient)?;
    let (photobooth, demo) = if args.remove_one::<bool>("async").unwrap_or_default() {
        runtime.block_on(future::zip(
            upload(&client, token.clone(), &photobooth),
            upload(&client, token.clone(), &demo),
        ))
    } else {
        (
            runtime.block_on(upload(&client, token.clone(), &photobooth)),
            runtime.block_on(upload(&client, token.clone(), &demo)),
        )
    };

    let photobooth = photobooth?;
    let demo = demo?;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PostDevlog<'a> {
        demo_video: &'a str,
        photobooth_video: &'a str,
        description: &'a str,
        neighbor: &'a str,
        app: &'a str,
    }

    runtime.block_on(async {
        client
            .post("https://neighborhood.hackclub.com/api/postDevlog")
            .json(&PostDevlog {
                demo_video: &demo,
                photobooth_video: &photobooth,
                description: &message,
                neighbor: &token,
                app: &app,
            })
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
            .map(|MessageResponse { message }| println!("{message}"))
    })
}
