use std::process::{Child, Command, Stdio};

pub struct Process {
    pub cmd: String,
    pub args: Vec<String>,
    pub process: Option<Child>,
    pub stream: bloom_logger::StreamConfig,
    pub should_terminate_on_death: bool,
    pub restart_on_path_change: Option<String>,
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
        restart_on_path_change: Option<String>,
    ) -> Self {
        Process {
            cmd: cmd.into(),
            args: args.into_iter().map(Into::into).collect::<Vec<String>>(),
            process: None,
            stream,
            should_terminate_on_death,
            restart_on_path_change,
        }
    }

    pub fn spawn(&mut self) -> Result<(), ()> {
        let mut child = Command::new(&self.cmd)
            .args(&self.args)
            .env("FORCE_COLOR", "1")
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

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

    pub fn restart(&mut self) -> Result<(), ()> {
        if let Some(child) = &mut self.process {
            if child.try_wait().map_err(|_| ())?.is_none() {
                child.kill().map_err(|_| ())?;
                child.wait().map_err(|_| ())?;
            }
        }

        self.process = None;

        self.spawn()?;

        Ok(())
    }
}
