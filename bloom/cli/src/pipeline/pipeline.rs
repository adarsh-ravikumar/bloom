use std::process::{Child, Command, Stdio};
use std::time::Duration;

type Action = Box<dyn Fn() -> Result<(), ()> + Send>;
type Shutdown = Box<dyn Fn(i32) + Send>;

pub struct Process {
    cmd: String,
    args: Vec<String>,
    process: Option<Child>,
    stream: bloom_logger::StreamConfig,
    should_terminate_on_death: bool,
}

impl Process {
    pub fn new(
        cmd: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
        stream: bloom_logger::StreamConfig,
        should_terminate_on_death: bool,
    ) -> Self {
        Process {
            cmd: cmd.into(),
            args: args.into_iter().map(Into::into).collect::<Vec<String>>(),
            process: None,
            stream,
            should_terminate_on_death,
        }
    }
}

pub struct Pipeline {
    processes: Vec<Process>,
    actions: Vec<Action>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            processes: Vec::new(),
            actions: Vec::new(),
        }
    }

    pub fn push_process(&mut self, process: Process) {
        self.processes.push(process);
    }

    pub fn push_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn monitor(mut processes: Vec<Process>, shutdown_handler: Shutdown) -> Result<(), ()> {
        loop {
            let mut should_kill = false;

            for process in &mut processes {
                let Some(child) = &mut process.process else {
                    continue;
                };

                if child.try_wait().map_err(|_| ())?.is_some() {
                    should_kill = process.should_terminate_on_death;
                }
            }

            if should_kill {
                Self::kill_all_processes(processes)?;
                let _ = shutdown_handler(0);
                std::process::exit(1);
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn kill_all_processes(mut processes: Vec<Process>) -> Result<(), ()> {
        for process in &mut processes {
            let Some(child) = &mut process.process else {
                continue;
            };

            if child.try_wait().map_err(|_| ())?.is_none() {
                let _ = child.kill();
            }
        }

        Ok(())
    }

    pub fn start(mut self, shutdown_handler: Shutdown) -> Result<(), ()> {
        for process in &mut self.processes {
            let mut child = Command::new(&process.cmd)
                .args(&process.args)
                .env("FORCE_COLOR", "1")
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|_| ())?;

            let stdout = child.stdout.take();

            if let Some(stdout) = stdout {
                let stream = process.stream;
                bloom_logger::spawn_log_stream(stream, bloom_logger::ProcessStream::Stdout(stdout));
            }

            let stderr = child.stderr.take();

            if let Some(stderr) = stderr {
                let stream = process.stream;
                bloom_logger::spawn_log_stream(stream, bloom_logger::ProcessStream::Stderr(stderr));
            }

            process.process = Some(child);
        }

        std::thread::spawn(move || {
            let _ = Self::monitor(self.processes, shutdown_handler);
        });

        for action in &mut self.actions {
            let _ = action().map_err(|_| ())?;
        }

        Ok(())
    }
}
