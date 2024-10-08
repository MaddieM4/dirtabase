use std::io::{self, Result, Write};

/// Controls where logs for a specific channel are routed to
#[derive(Copy, Clone)]
pub enum Policy {
    Stdout,
    Stderr,
    Silent,
}

pub struct Policies {
    /// Whether to print a message like "--- Import ---" before operations.
    pub opheader: Policy,

    /// Triad stack logging after each opheader.
    pub stack: Policy,

    /// Whether to print "--- [some shell command] ---" before running commands.
    pub cmd: Policy,
}

impl Default for Policies {
    fn default() -> Self {
        Self {
            opheader: Policy::Stdout,
            stack: Policy::Stdout,
            cmd: Policy::Stdout,
        }
    }
}

pub enum WriteBackend {
    RealStdout(io::Stdout),
    RealStderr(io::Stderr),
    ByteVector(Vec<u8>),
    Silent,
}
impl WriteBackend {
    pub fn recorded(&self) -> Option<&str> {
        match self {
            Self::ByteVector(v) => Some(
                std::str::from_utf8(&v).expect("WriteBackend.recorded failed to convert to str"),
            ),
            _ => None,
        }
    }
}
impl Write for WriteBackend {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        match self {
            Self::RealStdout(w) => w.write(bytes),
            Self::RealStderr(w) => w.write(bytes),
            Self::ByteVector(w) => w.write(bytes),
            Self::Silent => Ok(bytes.len()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Self::RealStdout(w) => w.flush(),
            Self::RealStderr(w) => w.flush(),
            Self::ByteVector(w) => w.flush(),
            Self::Silent => Ok(()),
        }
    }
}
impl From<io::Stdout> for WriteBackend {
    fn from(item: io::Stdout) -> Self {
        Self::RealStdout(item)
    }
}
impl From<io::Stderr> for WriteBackend {
    fn from(item: io::Stderr) -> Self {
        Self::RealStderr(item)
    }
}
impl From<Vec<u8>> for WriteBackend {
    fn from(item: Vec<u8>) -> Self {
        Self::ByteVector(item)
    }
}

pub struct Logger {
    pub stdout: WriteBackend,
    pub stderr: WriteBackend,
    pub pol: Policies,

    // Exists for dumb workaround reasons
    silent: WriteBackend,
}

impl Logger {
    pub fn new(stdout: impl Into<WriteBackend>, stderr: impl Into<WriteBackend>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            silent: WriteBackend::Silent,
            pol: Policies::default(),
        }
    }

    pub fn new_vec() -> Self {
        Self::new(vec![], vec![])
    }
    pub fn new_real() -> Self {
        Self::new(io::stdout(), io::stderr())
    }

    /// Only works with a vec logger. Will panic if used on real stdout/stderr.
    ///
    /// Useful for tests, which is the reason it exists!
    pub fn recorded<'a>(&'a self) -> (&'a str, &'a str) {
        (
            self.stdout.recorded().unwrap(),
            self.stderr.recorded().unwrap(),
        )
    }

    fn wb_for(&mut self, pol: Policy) -> &mut WriteBackend {
        match pol {
            Policy::Stdout => &mut self.stdout,
            Policy::Stderr => &mut self.stderr,
            Policy::Silent => &mut self.silent,
        }
    }

    pub fn opheader(&mut self) -> &mut WriteBackend {
        self.wb_for(self.pol.opheader)
    }
    pub fn stack(&mut self) -> &mut WriteBackend {
        self.wb_for(self.pol.stack)
    }
    pub fn cmd(&mut self) -> &mut WriteBackend {
        self.wb_for(self.pol.cmd)
    }
}

pub fn vec_logger() -> Logger {
    Logger::new_vec()
}

pub fn real_logger() -> Logger {
    Logger::new_real()
}

impl Default for Logger {
    fn default() -> Self {
        real_logger()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_stdout() -> Result<()> {
        let mut log = vec_logger();
        write!(log.stdout, "Writing to {}...", "stdout")?;
        assert_eq!(log.stdout.recorded().unwrap(), "Writing to stdout...");
        assert_eq!(log.stderr.recorded().unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_stderr() -> Result<()> {
        let mut log = vec_logger();
        write!(log.stderr, "Writing to {}...", "stderr")?;
        assert_eq!(log.stdout.recorded().unwrap(), "");
        assert_eq!(log.stderr.recorded().unwrap(), "Writing to stderr...");
        Ok(())
    }

    #[test]
    fn test_opheader() -> Result<()> {
        let mut log = vec_logger();
        write!(log.opheader(), "Writing to {}...", "somewhere")?;
        assert_eq!(log.stdout.recorded().unwrap(), "Writing to somewhere...");
        assert_eq!(log.stderr.recorded().unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_opheader_silent() -> Result<()> {
        let mut log = vec_logger();
        log.pol.opheader = Policy::Silent;

        write!(log.opheader(), "Writing to {}...", "somewhere")?;
        assert_eq!(log.stdout.recorded().unwrap(), "");
        assert_eq!(log.stderr.recorded().unwrap(), "");
        Ok(())
    }
}
