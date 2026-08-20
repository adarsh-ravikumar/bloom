use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use crate::pipeline::watch;
use crate::pipeline::{Process, ProcessCommand};

use bloom_api::BloomError;

type Action = Box<dyn Fn() -> Result<(), BloomError> + Send>;
type Shutdown = Box<dyn Fn(i32) + Send>;
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

    pub fn monitor(
        mut processes: Vec<Process>,
        rx: Receiver<ProcessCommand>,
        shutdown_handler: Shutdown,
    ) -> Result<(), BloomError> {
        loop {
            while let Ok(command) = rx.try_recv() {
                match command {
                    ProcessCommand::Restart(index) => {
                        processes[index].restart()?;
                    }
                }
            }
            let mut should_kill = false;

            for process in &mut processes {
                let Some(child) = &mut process.process else {
                    continue;
                };

                if child
                    .try_wait()
                    .map_err(|_| BloomError::WaitOnChildFailed)?
                    .is_some()
                {
                    should_kill = process.should_terminate_on_death;
                }
            }

            if should_kill {
                Self::kill_all_processes(processes)?;
                let _ = shutdown_handler(1);
                unreachable!();
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn kill_all_processes(mut processes: Vec<Process>) -> Result<(), BloomError> {
        for process in &mut processes {
            let Some(child) = &mut process.process else {
                continue;
            };

            if child
                .try_wait()
                .map_err(|_| BloomError::WaitOnChildFailed)?
                .is_none()
            {
                let _ = child.kill();
            }
        }

        Ok(())
    }

    pub fn start(mut self, shutdown_handler: Shutdown) -> Result<(), BloomError> {
        let (tx, rx) = channel();

        for index in 0..self.processes.len() {
            let process = self.processes.get_mut(index).unwrap();

            process.spawn()?;

            for path in process.watch_paths.iter() {
                let path = path.clone();
                std::thread::spawn({
                    let tx = tx.clone();
                    move || {
                        let _ = watch(path, move || {
                            tx.send(ProcessCommand::Restart(index)).ok();
                        });
                    }
                });
            }
        }

        std::thread::spawn(move || {
            let _ = Self::monitor(self.processes, rx, shutdown_handler);
        });

        for action in &mut self.actions {
            let _ = action()?;
        }

        Ok(())
    }
}
