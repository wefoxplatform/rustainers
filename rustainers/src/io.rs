use std::fmt::Display;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub enum StdIoKind {
    Out,
    Err,
}

impl Display for StdIoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Out => write!(f, "stdout"),
            Self::Err => write!(f, "stderr"),
        }
    }
}

/// An error during reading lines
#[derive(Debug, thiserror::Error)]
pub enum ReadLinesError {
    /// Fail to read the stream
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Fail to send the line
    #[error(transparent)]
    SenderError(#[from] mpsc::error::SendError<String>),
}

pub async fn read_lines<R>(
    reader: Option<R>,
    tx: mpsc::Sender<String>,
) -> Result<(), ReadLinesError>
where
    R: AsyncRead + Unpin,
{
    let Some(reader) = reader else {
        info!("No lines to read");
        return Ok(());
    };

    let buf_reader = BufReader::new(reader);
    let mut lines = buf_reader.lines();
    while let Some(line) = lines.next_line().await? {
        tx.send(line).await?;
    }

    Ok(())
}
