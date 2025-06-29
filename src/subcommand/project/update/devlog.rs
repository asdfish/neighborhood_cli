use {
    crate::{
        MainError,
        api::MessageResponse,
        cache::read_token,
        subcommand::project::update::{UploadApi, UploadVideo},
    },
    clap::ArgMatches,
    futures_lite::stream::{Stream, StreamExt},
    pin_project_lite::pin_project,
    reqwest::{Client, Response},
    serde::Serialize,
    std::{
        pin::{Pin, pin},
        task::{Context, Poll},
    },
    tokio::runtime,
};

pub trait FutureExt: Future {
    fn map<F, T>(self, f: F) -> impl Future<Output = T>
    where
        F: FnOnce(Self::Output) -> T,
        Self: Sized,
    {
        async move { f(self.await) }
    }
}
impl<T> FutureExt for T where T: Future {}

enum CompletionStatus {
    Both,
    Left,
    Right,
    None,
}
pin_project! {
    pub struct Both<L, R>
    where
        L: Future,
        R: Future<Output = L::Output>,
    {
        completion_status: CompletionStatus,
        #[pin]
        l: L,
        #[pin]
        r: R,
    }
}
impl<L, R> Both<L, R>
where
    L: Future,
    R: Future<Output = L::Output>,
{
    pub const fn new(l: L, r: R) -> Self {
        Self {
            completion_status: CompletionStatus::None,
            l,
            r,
        }
    }
}

impl<L, R> Stream for Both<L, R>
where
    L: Future,
    R: Future<Output = L::Output>,
{
    type Item = L::Output;

    fn poll_next(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.completion_status {
            CompletionStatus::Left => {
                *this.completion_status = CompletionStatus::Both;
                this.r.poll(ctx).map(Some)
            }
            CompletionStatus::Right => {
                *this.completion_status = CompletionStatus::Both;
                this.l.poll(ctx).map(Some)
            }
            CompletionStatus::Both => Poll::Ready(None),
            CompletionStatus::None => {
                if let output @ Poll::Ready(_) = this.l.poll(ctx).map(Some) {
                    *this.completion_status = CompletionStatus::Left;
                    return output;
                }
                if let output @ Poll::Ready(_) = this.r.poll(ctx).map(Some) {
                    *this.completion_status = CompletionStatus::Right;
                    return output;
                }

                Poll::Pending
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self.completion_status {
            CompletionStatus::Both => 0,
            CompletionStatus::Left | CompletionStatus::Right => 1,
            CompletionStatus::None => 2,
        };

        (len, Some(len))
    }
}

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

    enum Url {
        Photobooth,
        Demo,
    }

    let [photobooth, demo] = [
        (photobooth.as_str(), Url::Photobooth),
        (demo.as_str(), Url::Demo),
    ]
    .map(|(path, ty)| (UploadVideo::new(path).upload(&client, token.clone()), ty))
    .map(|(fut, ty)| fut.map(move |url| (url, ty)));

    let (photobooth, demo) = runtime.block_on(async move {
        let mut fut = Both::new(photobooth, demo);
        let mut fut = pin!(fut);
        let fst = match fut.next().await {
            Some((Ok(data), url)) => (data, url),
            Some((Err(error), _)) => return Err(error),
            _ => unreachable!(),
        };
        let snd = match fut.next().await {
            Some((Ok(data), url)) => (data, url),
            Some((Err(error), _)) => return Err(error),
            _ => unreachable!(),
        };

        match (fst, snd) {
            ((photobooth, Url::Photobooth), (demo, Url::Demo))
            | ((demo, Url::Demo), (photobooth, Url::Photobooth)) => Ok((photobooth, demo)),
            _ => unreachable!(),
        }
    })?;

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
