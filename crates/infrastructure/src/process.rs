//! App-owned subprocesses. Shutdown prevents new launches and reaps every live child.
use std::{
    io::{self, Read},
    process::{Child, Command, ExitStatus, Output},
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Duration,
};

#[derive(Default)]
struct Registry {
    closing: bool,
    children: Vec<Weak<Mutex<Child>>>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(Mutex::default)
}

pub struct ManagedProcess {
    child: Arc<Mutex<Child>>,
}
impl ManagedProcess {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        Self::spawn_in(command, registry())
    }
    fn spawn_in(command: &mut Command, registry: &Mutex<Registry>) -> io::Result<Self> {
        // Keep registration and the shutdown gate atomic with respect to launching.
        let mut registry = registry.lock().unwrap_or_else(|e| e.into_inner());
        if registry.closing {
            return Err(io::Error::other("Selah is shutting down"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let child = Arc::new(Mutex::new(command.spawn()?));
        registry.children.retain(|c| c.strong_count() > 0);
        registry.children.push(Arc::downgrade(&child));
        Ok(Self { child })
    }
    pub fn is_running(&self) -> bool {
        matches!(
            self.child
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .try_wait(),
            Ok(None)
        )
    }
    pub fn take_stdin(&self) -> Option<std::process::ChildStdin> {
        self.child
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .stdin
            .take()
    }
    pub fn wait(&self) -> io::Result<ExitStatus> {
        loop {
            // Do not hold the child lock during a blocking wait: shutdown must be able to kill it.
            if let Some(status) = self
                .child
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .try_wait()?
            {
                return Ok(status);
            }
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    pub fn wait_with_output(&self) -> io::Result<Output> {
        let (stdout, stderr) = {
            let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
            (child.stdout.take(), child.stderr.take())
        };
        std::thread::scope(|scope| {
            fn read(stream: Option<impl Read>) -> io::Result<Vec<u8>> {
                let mut bytes = Vec::new();
                if let Some(mut stream) = stream {
                    stream.read_to_end(&mut bytes)?;
                }
                Ok(bytes)
            }
            let out = scope.spawn(|| read(stdout));
            let err = scope.spawn(|| read(stderr));
            let status = self.wait()?;
            Ok(Output {
                status,
                stdout: out
                    .join()
                    .map_err(|_| io::Error::other("stdout reader failed"))??,
                stderr: err
                    .join()
                    .map_err(|_| io::Error::other("stderr reader failed"))??,
            })
        })
    }
}
fn stop(child: &Mutex<Child>) {
    let mut child = child.lock().unwrap_or_else(|e| e.into_inner());
    if matches!(child.try_wait(), Ok(Some(_))) {
        return;
    }
    #[cfg(unix)]
    unsafe {
        // Each child is a group leader; kill any descendants together with it.
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .output();
    }
    let _ = child.kill();
    let _ = child.wait();
}
impl Drop for ManagedProcess {
    fn drop(&mut self) {
        stop(&self.child);
    }
}
fn shutdown_registry(registry: &Mutex<Registry>) {
    let mut registry = registry.lock().unwrap_or_else(|e| e.into_inner());
    registry.closing = true;
    for child in registry.children.drain(..).filter_map(|c| c.upgrade()) {
        stop(&child);
    }
}
pub fn shutdown() {
    shutdown_registry(registry());
}
pub fn is_shutting_down() -> bool {
    registry().lock().unwrap_or_else(|e| e.into_inner()).closing
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn sleeper(registry: &Mutex<Registry>) -> ManagedProcess {
        ManagedProcess::spawn_in(Command::new("sleep").arg("30"), registry).unwrap()
    }
    #[test]
    fn shutdown_reaps_live_processes_and_prevents_late_launches() {
        let registry = Mutex::new(Registry::default());
        let first = sleeper(&registry);
        let second = sleeper(&registry);
        shutdown_registry(&registry);
        assert!(!first.is_running());
        assert!(!second.is_running());
        assert!(ManagedProcess::spawn_in(Command::new("sleep").arg("30"), &registry).is_err());
        shutdown_registry(&registry); // Safe to call from both close and exit events.
    }
    #[test]
    fn dropping_owner_reaps_child() {
        let registry = Mutex::new(Registry::default());
        let process = sleeper(&registry);
        let child = process.child.clone();
        drop(process);
        assert!(child.lock().unwrap().try_wait().unwrap().is_some());
    }
    #[test]
    fn output_is_drained_without_blocking_shutdown() {
        let registry = Mutex::new(Registry::default());
        let process = ManagedProcess::spawn_in(
            Command::new("sh")
                .args(["-c", "printf ready; sleep 30"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped()),
            &registry,
        )
        .unwrap();
        std::thread::scope(|scope| {
            let result = scope.spawn(|| process.wait_with_output());
            std::thread::sleep(Duration::from_millis(100));
            shutdown_registry(&registry);
            assert!(!result.join().unwrap().unwrap().status.success());
        });
    }
}
