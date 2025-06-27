use {
    crate::{cache::PathCache, MainError},
    clap::ArgMatches,
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "devlog" => {
            #[derive(Default, Deserialize, Serialize)]
            struct PublicDevlogForm {
                demo_video: PathBuf,
                photobooth_video: PathBuf,
                description: String,
                app: String,
            }

            let mut path_cache = PathCache::default();
            path_cache
                .set_token()
                .map(drop)
                .and_then(|_| path_cache.set_devlog())
                .map(drop)
                .and_then(|_| {
                    path_cache.get_token().and_then(|token| {
                        path_cache.get_devlog().map(move |devlog| (token, devlog))
                    })
                });

            // get_cache_file("devlog.toml")
            //     .map_err(MainError::GetCache)
            //     .map(|path| {

            //     });

            Ok(())
        }
        _ => unreachable!(),
    }
}
