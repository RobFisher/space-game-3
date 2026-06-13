use std::{env, error::Error, ffi::OsString, fmt};

use crate::app::DEFAULT_SERVER_URL;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientArgs {
    pub server_url: String,
    pub mode: ClientMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientMode {
    Tui,
    Plain { command: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    MissingValue(&'static str),
    UnexpectedArgument(String),
    DuplicateServerUrl,
}

impl ClientArgs {
    pub fn parse_env() -> Result<Self, ArgsError> {
        Self::parse_from(env::args_os().skip(1))
    }

    pub fn parse_from<I, S>(args: I) -> Result<Self, ArgsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut plain = false;
        let mut command = None;
        let mut server_url = None;
        let mut args = args.into_iter().map(Into::into);

        while let Some(arg) = args.next() {
            let arg = arg.to_string_lossy().into_owned();
            match arg.as_str() {
                "--plain" => plain = true,
                "--command" | "-c" => {
                    command = Some(next_value(&mut args, "--command")?);
                }
                "--server" | "--url" => {
                    set_server_url(&mut server_url, next_value(&mut args, "--server")?)?;
                }
                "--help" | "-h" => return Err(ArgsError::UnexpectedArgument(arg)),
                value if value.starts_with('-') => {
                    return Err(ArgsError::UnexpectedArgument(arg));
                }
                value => {
                    set_server_url(&mut server_url, value.to_string())?;
                }
            }
        }

        Ok(Self {
            server_url: server_url.unwrap_or_else(|| DEFAULT_SERVER_URL.to_string()),
            mode: if plain {
                ClientMode::Plain { command }
            } else {
                ClientMode::Tui
            },
        })
    }
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue(flag) => write!(f, "missing value for {flag}"),
            Self::UnexpectedArgument(arg) => write!(f, "unexpected argument: {arg}"),
            Self::DuplicateServerUrl => write!(f, "server URL was provided more than once"),
        }
    }
}

impl Error for ArgsError {}

fn next_value<I>(args: &mut I, flag: &'static str) -> Result<String, ArgsError>
where
    I: Iterator<Item = OsString>,
{
    args.next()
        .map(|value| value.to_string_lossy().into_owned())
        .ok_or(ArgsError::MissingValue(flag))
}

fn set_server_url(server_url: &mut Option<String>, value: String) -> Result<(), ArgsError> {
    if server_url.replace(value).is_some() {
        return Err(ArgsError::DuplicateServerUrl);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_tui_with_default_server() {
        let args = ClientArgs::parse_from(std::iter::empty::<&str>()).unwrap();

        assert_eq!(args.server_url, DEFAULT_SERVER_URL);
        assert_eq!(args.mode, ClientMode::Tui);
    }

    #[test]
    fn parses_plain_with_command() {
        let args = ClientArgs::parse_from(["--plain", "--command", "objects"]).expect("parse args");

        assert_eq!(args.server_url, DEFAULT_SERVER_URL);
        assert_eq!(
            args.mode,
            ClientMode::Plain {
                command: Some("objects".to_string())
            }
        );
    }

    #[test]
    fn parses_server_url_override_flag() {
        let args = ClientArgs::parse_from(["--server", "ws://127.0.0.1:5000/ws"]).unwrap();

        assert_eq!(args.server_url, "ws://127.0.0.1:5000/ws");
        assert_eq!(args.mode, ClientMode::Tui);
    }

    #[test]
    fn parses_positional_server_url_override() {
        let args = ClientArgs::parse_from(["--plain", "ws://127.0.0.1:5000/ws"]).unwrap();

        assert_eq!(args.server_url, "ws://127.0.0.1:5000/ws");
        assert_eq!(args.mode, ClientMode::Plain { command: None });
    }

    #[test]
    fn rejects_missing_command_value() {
        assert_eq!(
            ClientArgs::parse_from(["--plain", "--command"]).unwrap_err(),
            ArgsError::MissingValue("--command")
        );
    }
}
