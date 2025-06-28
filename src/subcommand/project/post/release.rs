use {
    crate::{MainError, cache::PathCache, env},
    clap::ArgMatches,
    serde::{Deserialize, Deserializer, Serialize},
    std::{
        borrow::Cow,
        ffi::OsString,
        fs::{self, DirBuilder, File, OpenOptions},
        io::{Write, stdin},
        path::PathBuf,
        process::{Command, Stdio},
    },
    tempfile::tempdir,
    toml_edit::{Date, DocumentMut, TomlError},
};

enum Either<L, R> {
    Left(L),
    Right(R),
}
impl<L, R> Either<L, R> {
    // fn right_or_else<F>(self, f: F) -> R
    // where
    //     F: FnOnce(L) -> R {
    //     match self {
    //         Self::Left(l)
    //         Self::Right(r) => r,
    //     }
    // }
    fn converge<F, G, T>(self, f: F, g: G) -> T
    where
        F: FnOnce(L) -> T,
        G: FnOnce(R) -> T,
    {
        match self {
            Self::Left(l) => f(l),
            Self::Right(r) => g(r),
        }
    }
}

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
struct Cache {
    #[serde(deserialize_with = "deserialize_non_empty_string", skip_deserializing)]
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

const INITIAL_CACHE: &str = r#"# All the fields, unless specified otherwise should contain a value
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

fn validate(cache: &str) -> Result<DocumentMut, TomlError> {
    cache.parse::<DocumentMut>().and_then(|document| {
        toml_edit::de::from_str::<Cache>(&cache)
            .map(move |_| document)
            .map_err(TomlError::from)
    })
}

pub fn execute(mut args: ArgMatches, name: &str, _async_upload: bool) -> Result<(), MainError> {
    PathCache::default()
        .into_release()
        .map_err(MainError::GetCache)
        .and_then(|release| {
            if !release.is_dir() {
                match DirBuilder::new().recursive(true).create(&release) {
                    Ok(()) => Ok(release),
                    Err(error) => Err(MainError::CreateDirectory(error, release)),
                }
            } else {
                Ok(release)
            }
        })
        .map(|mut cache| {
            cache.push(name);
            cache.set_extension("toml");
            cache
        })
        .and_then(|cache| {
            if !cache.exists() || args.remove_one("edit").unwrap_or_default() {
                let contents = if !cache.exists() || args.remove_one("reset").unwrap_or_default() {
                    Ok(Cow::Borrowed(INITIAL_CACHE))
                } else {
                    fs::read_to_string(&cache).map(Cow::Owned)
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

                            let cache = fs::read_to_string(&path)
                                .map_err(|error| MainError::ReadFile(error, path.clone()))?;

                            match cache.parse::<DocumentMut>().and_then(|document| {
                                toml_edit::de::from_str::<Cache>(&cache)
                                    .map(move |_| document)
                                    .map_err(TomlError::from)
                            }) {
                                Ok(document) => {
                                    let _ = fs::write(&path, cache);
                                    break Ok(Some(document));
                                }
                                Err(error) => {
                                    eprintln!("{error}");
                                    loop {
                                        eprintln!("Retry: (yes/no)?: ");
                                        line.clear();
                                        stdin()
                                            .read_line(&mut line)
                                            .map_err(MainError::ReadLine)?;
                                        match line.trim() {
                                            "y" | "yes" => continue,
                                            "n" | "no" => return Ok(None),
                                            response => eprintln!("unknown option `{response}`"),
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => Err(MainError::ReadFile(error, cache)),
                }
            } else {
                fs::read_to_string(&cache)
                    .map_err(|error| MainError::ReadFile(error, cache))
                    .and_then(|cache| validate(&cache).map_err(MainError::ParseCache))
                    .map(Some)
            }
        })
        .map(|mut document| {})
    // .map(|cache| cache.convert())
    // .and_then(|cache| toml::from_str::<Cache>(&cache).map_err(MainError::ParseCache))
    // .map(|mut cache| {
    //     cache.changes_made = args.remove_one("message").unwrap();
    //     cache
    // })
    // .map(|cache| eprintln!("{cache:?}"))
}
