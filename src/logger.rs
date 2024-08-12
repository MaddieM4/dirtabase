use std::io::{Result, Write};

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
}

impl Default for Policies {
    fn default() -> Self {
        Self {
            opheader: Policy::Stdout,
        }
    }
}

pub enum LogWriter<'a, OUT, ERR>
where
    OUT: Write,
    ERR: Write,
{
    Stdout(&'a mut OUT),
    Stderr(&'a mut ERR),
    Silent,
}
impl<'a, OUT, ERR> Write for LogWriter<'a, OUT, ERR>
where
    OUT: Write,
    ERR: Write,
{
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        match self {
            Self::Stdout(w) => w.write(bytes),
            Self::Stderr(w) => w.write(bytes),
            Self::Silent => Ok(bytes.len()),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Stdout(w) => w.flush(),
            Self::Stderr(w) => w.flush(),
            Self::Silent => Ok(()),
        }
    }
}

pub struct Logger<'a, OUT, ERR>
where
    OUT: Write,
    ERR: Write,
{
    pub stdout: &'a mut OUT,
    pub stderr: &'a mut ERR,
    pub pol: Policies,
}

impl<'a, OUT, ERR> Logger<'a, OUT, ERR>
where
    OUT: Write,
    ERR: Write,
{
    pub fn new(stdout: &'a mut OUT, stderr: &'a mut ERR) -> Self {
        Self {
            stdout: stdout,
            stderr: stderr,
            pol: Policies::default(),
        }
    }

    fn lw_stdout(&mut self) -> LogWriter<OUT, ERR> {
        LogWriter::Stdout(self.stdout)
    }
    fn lw_stderr(&mut self) -> LogWriter<OUT, ERR> {
        LogWriter::Stderr(self.stderr)
    }
    fn lw_silent(&mut self) -> LogWriter<OUT, ERR> {
        LogWriter::Silent
    }
    fn lw_for(&mut self, pol: Policy) -> LogWriter<OUT, ERR> {
        match pol {
            Policy::Stdout => self.lw_stdout(),
            Policy::Stderr => self.lw_stderr(),
            Policy::Silent => self.lw_silent(),
        }
    }

    pub fn opheader(&mut self) -> LogWriter<OUT, ERR> {
        self.lw_for(self.pol.opheader)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lwstdout() -> Result<()> {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        write!(log.lw_stdout(), "Writing to {}...", "stdout")?;
        assert_eq!(&String::from_utf8(stdout).unwrap(), "Writing to stdout...");
        assert_eq!(&String::from_utf8(stderr).unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_lwstderr() -> Result<()> {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        write!(log.lw_stderr(), "Writing to {}...", "stderr")?;
        assert_eq!(&String::from_utf8(stdout).unwrap(), "");
        assert_eq!(&String::from_utf8(stderr).unwrap(), "Writing to stderr...");
        Ok(())
    }

    #[test]
    fn test_opheader() -> Result<()> {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        write!(log.opheader(), "Writing to {}...", "somewhere")?;
        assert_eq!(
            &String::from_utf8(stdout).unwrap(),
            "Writing to somewhere..."
        );
        assert_eq!(&String::from_utf8(stderr).unwrap(), "");
        Ok(())
    }

    #[test]
    fn test_opheader_silent() -> Result<()> {
        let mut stdout: Vec<u8> = vec![];
        let mut stderr: Vec<u8> = vec![];
        let mut log = Logger::new(&mut stdout, &mut stderr);
        log.pol.opheader = Policy::Silent;

        write!(log.opheader(), "Writing to {}...", "somewhere")?;
        assert_eq!(&String::from_utf8(stdout).unwrap(), "");
        assert_eq!(&String::from_utf8(stderr).unwrap(), "");
        Ok(())
    }
}
