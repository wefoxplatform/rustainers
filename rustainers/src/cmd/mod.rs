use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::fmt::{self, Display};
use std::path::Path;
use std::process::{ExitStatus, Output, Stdio};
use tokio::sync::mpsc;
use tracing::{debug, warn};

mod error;
use crate::io::{read_lines, StdIoKind};

pub use self::error::*;

#[derive(Debug, Clone)]
pub(crate) struct Cmd<'a> {
    command: &'a str,
    args: Vec<String>,
    dir: Option<&'a Path>,
    env: IndexMap<String, String>,
    ignore_stderr: bool,
}

impl<'a> Cmd<'a> {
    pub(crate) fn new(command: &'a str) -> Self {
        Self {
            command,
            args: vec![],
            dir: None,
            env: IndexMap::new(),
            ignore_stderr: false,
        }
    }

    pub(crate) fn with_dir(&mut self, path: &'a Path) {
        self.dir = Some(path);
    }

    pub(crate) fn set_env(&mut self, env: IndexMap<String, String>) {
        self.env = env;
    }

    pub(crate) fn ignore_stderr(&mut self) {
        self.ignore_stderr = true;
    }

    pub(crate) fn push_arg(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
    }

    pub(crate) fn push_args<S>(&mut self, args: impl IntoIterator<Item = S>)
    where
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
    }

    fn handle_output(&self, output: std::io::Result<Output>) -> Result<Output, CommandError> {
        let output = match output {
            Ok(output) => output,
            Err(source) => {
                return Err(CommandError::CommandProcessError {
                    command: self.to_string(),
                    source,
                })
            }
        };
        if !self.ignore_stderr && !output.stderr.is_empty() {
            let err = String::from_utf8_lossy(&output.stderr);
            let command = self.to_string();
            warn!(%command, "stderr\n{err}");
        }

        if output.status.success() {
            Ok(output)
        } else {
            let command = self.to_string();
            Err(CommandError::CommandFail { command, output })
        }
    }

    fn handle_json<T>(&self, output: Output) -> Result<T, CommandError>
    where
        T: DeserializeOwned,
    {
        let result =
            serde_json::from_slice(&output.stdout).map_err(|source| CommandError::SerdeError {
                command: format!("{self}"),
                output,
                source,
            })?;
        Ok(result)
    }

    fn handle_json_stream<T>(&self, output: Output) -> Result<Vec<T>, CommandError>
    where
        T: DeserializeOwned,
    {
        let stream = serde_json::Deserializer::from_slice(&output.stdout).into_iter::<T>();
        stream
            .collect::<Result<_, _>>()
            .map_err(|source| CommandError::SerdeError {
                command: format!("{self}"),
                output,
                source,
            })
    }
}

// Blocking API
impl<'a> Cmd<'a> {
    fn output_blocking(&self) -> Result<Output, CommandError> {
        debug!("Running blocking command\n{self}");
        let mut c = std::process::Command::new(self.command);
        c.envs(&self.env);
        if let Some(dir) = self.dir {
            c.current_dir(dir);
        }
        let output = c.args(&self.args).output();
        self.handle_output(output)
    }

    pub(super) fn result_blocking(self) -> Result<String, CommandError> {
        let output = self.output_blocking()?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    }

    pub(super) fn json_blocking<T>(self) -> Result<T, CommandError>
    where
        T: DeserializeOwned,
    {
        let output = self.output_blocking()?;
        self.handle_json(output)
    }

    pub(super) fn status_blocking(self) -> Result<ExitStatus, CommandError> {
        let output = self.output_blocking()?;
        Ok(output.status)
    }
}

// Async API
impl<'a> Cmd<'a> {
    async fn output(&self) -> Result<Output, CommandError> {
        debug!("Running command\n{self}");
        let mut c = tokio::process::Command::new(self.command);
        c.envs(&self.env);
        if let Some(dir) = self.dir {
            c.current_dir(dir);
        }
        let output = c.args(&self.args).output().await;
        self.handle_output(output)
    }

    pub(super) async fn watch_io(
        &self,
        io: StdIoKind,
        tx: mpsc::Sender<String>,
    ) -> Result<Output, CommandError> {
        debug!("Running command\n{self}");
        let mut c = tokio::process::Command::new(self.command);
        c.envs(&self.env);
        if let Some(dir) = self.dir {
            c.current_dir(dir);
        }
        c.stdout(Stdio::piped());
        c.stderr(Stdio::piped());

        let mut child = c
            .args(&self.args)
            .spawn()
            .map_err(|source| CommandError::IoError {
                command: self.to_string(),
                source,
            })?;

        let result = match io {
            StdIoKind::Out => read_lines(child.stdout.take(), tx).await,
            StdIoKind::Err => read_lines(child.stderr.take(), tx).await,
        };
        if let Err(source) = result {
            return Err(CommandError::CommandWatchFail {
                command: self.to_string(),
                source,
            });
        }

        let output = child.wait_with_output().await;
        self.handle_output(output)
    }

    pub(super) async fn result(&self) -> Result<String, CommandError> {
        let output = self.output().await?;
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    }

    pub(super) async fn json<T>(self) -> Result<T, CommandError>
    where
        T: DeserializeOwned,
    {
        let output = self.output().await?;
        self.handle_json(output)
    }

    pub(super) async fn json_stream<T>(self) -> Result<Vec<T>, CommandError>
    where
        T: DeserializeOwned,
    {
        let output = self.output().await?;
        self.handle_json_stream(output)
    }

    pub(super) async fn status(self) -> Result<ExitStatus, CommandError> {
        let output = self.output().await?;
        Ok(output.status)
    }
}

pub(crate) fn escape_arg(arg: &str) -> Cow<'_, str> {
    if arg.contains(' ') {
        Cow::Owned(format!("\"{arg}\""))
    } else {
        Cow::Borrowed(arg)
    }
}

impl<'a> Display for Cmd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.command)?;
        for arg in &self.args {
            let arg = escape_arg(arg);
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}
