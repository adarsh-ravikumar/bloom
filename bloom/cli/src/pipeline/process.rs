use std::process::{Child, Command, Stdio};

use bloom_api::BloomError;

pub struct Process {
    pub cmd: String,
    pub args: Vec<String>,
    pub process: Option<Child>,
    pub stream: bloom_logger::StreamConfig,
    pub should_terminate_on_death: bool,
    pub watch_paths: Vec<String>,
}

pub enum ProcessCommand {
    Restart(usize),
}

impl Process {
    pub fn new(
        cmd: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
        stream: bloom_logger::StreamConfig,
        should_terminate_on_death: bool,
        watch_paths: Vec<String>,
    ) -> Self {
        Process {
            cmd: cmd.into(),
            args: args.into_iter().map(Into::into).collect::<Vec<String>>(),
            process: None,
            stream,
            should_terminate_on_death,
            watch_paths,
        }
    }

    pub fn spawn(&mut self) -> Result<(), BloomError> {
        let mut child = Command::new(&self.cmd)
            .args(&self.args)
            .env("FORCE_COLOR", "1")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| BloomError::SpawnChildFailed)?;

        let stdout = child.stdout.take();

        if let Some(stdout) = stdout {
            let stream = self.stream;
            bloom_logger::spawn_log_stream(stream, bloom_logger::ProcessStream::Stdout(stdout));
        }

        let stderr = child.stderr.take();

        if let Some(stderr) = stderr {
            let stream = self.stream;
            bloom_logger::spawn_log_stream(stream, bloom_logger::ProcessStream::Stderr(stderr));
        }

        self.process = Some(child);

        Ok(())
    }

    pub fn restart(&mut self) -> Result<(), BloomError> {
        if let Some(child) = &mut self.process {
            if child
                .try_wait()
                .map_err(|_| BloomError::WaitOnChildFailed)?
                .is_none()
            {
                child.kill().map_err(|_| BloomError::KillChildFailed)?;
                child.wait().map_err(|_| BloomError::WaitOnChildFailed)?;
            }
        }

        self.process = None;

        self.spawn()?;

        Ok(())
    }
}
