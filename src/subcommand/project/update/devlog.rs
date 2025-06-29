use {
    crate::{
        MainError,
        api::MessageResponse,
        cache::read_token,
        subcommand::project::update::{UploadApi, UploadVideo},
    },
    clap::ArgMatches,
    futures_lite::future,
    reqwest::{Client, Response},
    serde::Serialize,
    tokio::runtime,
};

pub fn execute(mut args: ArgMatches, name: &str, message: &str) -> Result<(), MainError> {
    let photobooth = args.remove_one::<String>("photobooth").unwrap();
    let demo = args.remove_one::<String>("demo").unwrap();

    let token = read_token()?;

    let runtime = runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(MainError::CreateRuntime)?;

    let client = Client::builder().build().map_err(MainError::CreateClient)?;
    let photobooth = UploadVideo::new(&photobooth);
    let demo = UploadVideo::new(&demo);
    let photobooth = photobooth.upload(&client, token.clone());
    let demo = demo.upload(&client, token.clone());
    let (photobooth, demo) = runtime.block_on(future::zip(photobooth, demo));

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
                description: message,
                neighbor: &token,
                app: name,
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
