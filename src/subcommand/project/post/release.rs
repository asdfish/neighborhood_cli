use {
    crate::{
        api::MessageResponse,
        MainError,
        cache::PathCache,
        env,
        subcommand::project::post::{UploadApi, UploadImages},
    },
    clap::ArgMatches,
    reqwest::{Client, Response},
    serde::{Deserialize, Deserializer, Serialize},
    std::{
        borrow::Cow,
        ffi::OsString,
        fs::{self, DirBuilder, File, OpenOptions},
        io::{Write, stdin},
        iter,
        path::PathBuf,
        process::{Command, Stdio},
    },
    tempfile::tempdir,
    tokio::runtime,
    toml_edit::{Date, DocumentMut, Formatted, Item, TomlError, Value},
};

const ERROR: &str = "string cannot be empty";

pub fn deserialize_vec_non_empty_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<String>::deserialize(deserializer)?;

    if let Some((i, _)) = vec.iter().enumerate().find(|(_, string)| string.is_empty()) {
        Err(<D::Error as serde::de::Error>::custom(format!(
            "{ERROR}: screenshots[{i}]"
        )))
    } else {
        Ok(vec)
    }
}

pub fn deserialize_non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    if string.is_empty() {
        Err(<D::Error as serde::de::Error>::custom(ERROR))
    } else {
        Ok(string)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseConfig {
    #[serde(skip_deserializing)]
    token: String,

    #[serde(skip_deserializing)]
    changes_made: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    code_url: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    description: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    playable_url: String,

    #[serde(deserialize_with = "deserialize_vec_non_empty_string")]
    screenshots: Vec<String>,
    #[serde(
        deserialize_with = "deserialize_vec_non_empty_string",
        skip_serializing
    )]
    new_screenshot_paths: Vec<String>,

    #[serde(deserialize_with = "deserialize_non_empty_string")]
    address_line_1: String,
    address_line_2: String,
    birthday: Date,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    city: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    country: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    email: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    github_username: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    first_name: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    last_name: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    state_province: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    zip_code: String,

    #[serde(deserialize_with = "deserialize_non_empty_string")]
    how_can_we_improve: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    how_did_you_hear: String,
    #[serde(deserialize_with = "deserialize_non_empty_string")]
    what_are_we_doing_well: String,
}

const INITIAL_RELEASE_CONFIG: &str = r#"# All the fields, unless specified otherwise should contain a value
# Project
codeUrl = "" # The link to your repository
description = "" # Project description
playableUrl = "" # Link that showcases your project. Can be a live demo like a website or a release link

# Screenshots
# An array of urls pointing to screenshots
# You should not use this to add new screenshots, instead you should only use this to remove screenshots
screenshots = []
# An array of paths that will be uploaded and then added to the screenshot array above
newScreenshotPaths = []

# Personal information
addressLine1 = ""
addressLine2 = "" # Optional
birthday = 2000-01-01 # ISO-8601 format (YYYY-MM-DD)
city = ""
country = "" # Country code
email = "" # The email used to sign up into neighborhood
githubUsername = ""
firstName = ""
lastName = ""
stateProvince = ""
zipCode = ""

# Telemetry
howCanWeImprove = ""
howDidYouHear = ""
whatAreWeDoingWell = """#;

fn validate(release_config: &str) -> Result<DocumentMut, TomlError> {
    release_config.parse::<DocumentMut>().and_then(|document| {
        toml_edit::de::from_str::<ReleaseConfig>(&release_config)
            .map(move |_| document)
            .map_err(TomlError::from)
    })
}

pub fn execute(mut args: ArgMatches, name: &str, async_upload: bool) -> Result<(), MainError> {
    let mut path_cache = PathCache::default();
    path_cache.set_release()?;
    path_cache.set_token()?;
    let token = path_cache.read_token()?;
    let release_config = path_cache.get_release();
    let mut release_config = release_config
        .map_err(MainError::GetCache)
        .and_then(|release| {
            if !release.is_dir() {
                match DirBuilder::new().recursive(true).create(&release) {
                    Ok(()) => Ok(release.to_path_buf()),
                    Err(error) => Err(MainError::CreateDirectory(error, release.to_path_buf())),
                }
            } else {
                Ok(release.to_path_buf())
            }
        })?;
    release_config.push(name);
    release_config.set_extension("toml");

    if !release_config.exists() || args.remove_one("edit").unwrap_or_default() {
        let contents = if !release_config.exists() || args.remove_one("reset").unwrap_or_default() {
            Ok(Cow::Borrowed(INITIAL_RELEASE_CONFIG))
        } else {
            fs::read_to_string(&release_config).map(Cow::Owned)
        };

        match contents {
            Ok(contents) => {
                let dir = tempdir().map_err(MainError::CreateTempDir)?;

                let mut path = PathBuf::from(dir.path());
                path.push(name);
                path.set_extension("toml");

                fs::write(&path, contents.as_ref())
                    .map_err(|error| MainError::WriteFile(error, path.clone()))?;

                let command = args
                    .remove_one::<String>("editor")
                    .map(OsString::from)
                    .map(Cow::Owned)
                    .or_else(|| {
                        // SAFETY: this is single threaded
                        unsafe { env::var(c"VISUAL") }
                            .or_else(|| unsafe { env::var(c"EDITOR") })
                            .map(Cow::Borrowed)
                    })
                    .ok_or(MainError::NoEditor)?;

                let mut command = Command::new(command);
                command
                    .stdin(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .args(args.remove_many::<String>("arg").unwrap_or_default())
                    .arg(&path);

                let mut line = String::with_capacity(3);
                loop {
                    command.output().map_err(|error| {
                        MainError::ExecuteCommand(error, format!("{command:?}"))
                    })?;

                    let release_config = fs::read_to_string(&path)
                        .map_err(|error| MainError::ReadFile(error, path.clone()))?;

                    match release_config.parse::<DocumentMut>().and_then(|document| {
                        toml_edit::de::from_str::<ReleaseConfig>(&release_config)
                            .map(move |_| document)
                            .map_err(TomlError::from)
                    }) {
                        Ok(document) => {
                            let _ = fs::write(&path, release_config);
                            break Ok(document);
                        }
                        Err(error) => {
                            eprintln!("{error}");
                            loop {
                                eprintln!("Retry: (yes/no)?: ");
                                line.clear();
                                stdin().read_line(&mut line).map_err(MainError::ReadLine)?;
                                match line.trim() {
                                    "y" | "yes" => continue,
                                    "n" | "no" => return Ok(()),
                                    response => eprintln!("unknown option `{response}`"),
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => Err(MainError::ReadFile(error, release_config.clone())),
        }
    } else {
        fs::read_to_string(&release_config)
            .map_err(|error| MainError::ReadFile(error, release_config.clone()))
            .and_then(|release_config| {
                validate(&release_config).map_err(MainError::ParseReleaseConfig)
            })
    }
    .and_then(|mut document| {
        let Some(Item::Value(Value::Array(new_screenshot_paths))) =
            document.remove("newScreenshotPaths")
        else {
            unreachable!()
        };

        let request = UploadImages::new(
            new_screenshot_paths
                .into_iter()
                .flat_map(|val| match val {
                    Value::String(string) => Some(string),
                    _ => None,
                })
                .map(Formatted::into_value),
        );

        let runtime = runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(MainError::CreateRuntime)?;
        let client = Client::builder().build().map_err(MainError::CreateClient)?;
        let urls = runtime.block_on(request.upload(&client, token))?;

        document
            .as_item_mut()
            .as_table_mut()
            .and_then(|table| table.get_mut("screenshots"))
            .map(|screenshots| {
                if let Item::Value(Value::Array(screenshots)) = screenshots {
                    urls.into_iter()
                        .filter(|url| !url.is_empty())
                        .for_each(|url| screenshots.push(url))
                }
            });

        fs::write(&release_config, document.to_string());
        let document = toml_edit::de::from_document::<ReleaseConfig>(document)
            .map_err(TomlError::from)
            .map_err(MainError::ParseReleaseConfig)?;

        runtime.block_on(async {
            client
                .post("https://neighborhood.hackclub.com/api/shipApp")
                .json(&document)
                .send()
                .await
                .and_then(Response::error_for_status)
                .map_err(MainError::ExecuteRequest)?
                .text()
                .await
                .map_err(MainError::ExecuteRequest)
                .and_then(|response| {
                    serde_json::from_str(&response)
                        .map_err(|error| MainError::DecodeResponse(error, response.to_string()))
                })
                .map(|MessageResponse { message }| {
                    eprintln!("{message}");
                })
        });

        Ok(())
    })
}
