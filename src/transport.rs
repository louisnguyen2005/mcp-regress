use crate::config::Server;
use serde_json::Value;
use std::{future::Future, pin::Pin, process::Stdio};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

pub type IoFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'a>>;

/// A bounded message channel. Protocol semantics live in `Client`, not here.
pub trait Transport: Send {
    fn send(&mut self, message: Value) -> IoFuture<'_, ()>;
    fn receive(&mut self) -> IoFuture<'_, Value>;
    fn close(&mut self) -> IoFuture<'_, ()>;
}

pub struct StdioTransport {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    frame_limit: usize,
    remaining: usize,
    #[cfg(unix)]
    group: Option<i32>,
}

impl StdioTransport {
    pub fn spawn(server: &Server) -> Result<Self, String> {
        let mut command = Command::new(&server.command);
        if server.env_clear {
            command.env_clear();
        }
        command
            .args(&server.args)
            .envs(&server.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command
            .spawn()
            .map_err(|_| "cannot start configured server".to_string())?;
        #[cfg(unix)]
        let group = Some(child.id().ok_or("missing process ID")? as i32);
        let input = child.stdin.take().ok_or("missing stdin")?;
        let output = BufReader::new(child.stdout.take().ok_or("missing stdout")?);
        Ok(Self {
            child,
            input,
            output,
            frame_limit: server.max_frame_bytes,
            remaining: server.max_output_bytes,
            #[cfg(unix)]
            group,
        })
    }

    fn terminate(&mut self) {
        #[cfg(unix)]
        // The child is a new process group leader. Never signal our own group.
        if let Some(group) = self.group.take() {
            unsafe {
                libc::kill(-group, libc::SIGKILL);
            }
        }
        let _ = self.child.start_kill();
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Reads incrementally and rejects excess bytes before extending the frame buffer.
pub async fn read_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let mut frame = Vec::new();
    loop {
        let available = reader.fill_buf().await.map_err(|_| "stdout read failed")?;
        if available.is_empty() {
            return Err("server closed stdout before a complete frame".into());
        }
        let count = available
            .iter()
            .position(|b| *b == b'\n')
            .map_or(available.len(), |i| i + 1);
        if frame.len().saturating_add(count) > limit {
            return Err("MCP frame exceeds byte limit".into());
        }
        let done = available[count - 1] == b'\n';
        frame.extend_from_slice(&available[..count]);
        reader.consume(count);
        if done {
            return Ok(frame);
        }
    }
}

impl Transport for StdioTransport {
    fn send(&mut self, message: Value) -> IoFuture<'_, ()> {
        Box::pin(async move {
            let mut bytes = serde_json::to_vec(&message).map_err(|_| "request encoding failed")?;
            bytes.push(b'\n');
            if bytes.len() > self.frame_limit {
                return Err("request exceeds frame limit".into());
            }
            self.input
                .write_all(&bytes)
                .await
                .map_err(|_| "stdin write failed".into())
        })
    }
    fn receive(&mut self) -> IoFuture<'_, Value> {
        Box::pin(async move {
            let bytes = read_frame(&mut self.output, self.frame_limit.min(self.remaining)).await?;
            self.remaining -= bytes.len();
            serde_json::from_slice(&bytes).map_err(|_| "invalid JSON on server stdout".into())
        })
    }
    fn close(&mut self) -> IoFuture<'_, ()> {
        Box::pin(async move {
            self.terminate();
            self.child
                .wait()
                .await
                .map_err(|_| "could not reap server")?;
            Ok(())
        })
    }
}
