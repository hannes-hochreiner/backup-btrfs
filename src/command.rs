use anyhow::{anyhow, Context as _, Result};
use std::process;
#[cfg(test)]
mod tests;

pub trait Command {
    /// Runs a command in the provided context
    ///
    /// * `command` - command string
    /// * `context` - either a local or a remote context
    ///
    fn run(&mut self, command: &str, context: &Context) -> Result<String>;

    /// Runs several commands piping stdout of one command into stdin of the next
    ///
    /// * `commands` - a vector of tuples of command strings and contexts
    ///
    fn run_piped(&mut self, commands: &Vec<(&str, &Context)>) -> Result<String>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum Context {
    /// Local context
    ///
    /// * `user` - name of the user who will execute the command
    ///
    Local { user: String },
    /// Remote context
    ///
    /// * `host` - name of the remote host
    /// * `user` - name of the user on the remote host
    /// * `identity` - path and filename of the ssh identity file
    ///
    Remote {
        host: String,
        user: String,
        identity: String,
    },
}

pub struct CommandSystem {}

impl CommandSystem {
    fn run_piped(&mut self, commands: &Vec<(&str, &Context)>) -> Result<String> {
        let mut child: Option<process::Child> = None;

        for (command, context) in commands {
            match child {
                Some(mut c) => {
                    child = Some(self.run_single(command, context, Some(&mut c))?);
                }
                None => {
                    child = Some(self.run_single(command, context, None)?);
                }
            }
        }

        let output = child
            .ok_or(anyhow!("error executing command"))?
            .wait_with_output()?;
        let output = check_output(&output)
            .context("output of command to delete a snapshot contained an error")?;

        Ok(String::from_utf8(output)?)
    }

    fn run_single(
        &mut self,
        command: &str,
        context: &Context,
        pre: Option<&mut process::Child>,
    ) -> Result<process::Child> {
        let mut com = match context {
            Context::Local { user } => {
                let mut com = process::Command::new("sudo");
                com.arg("-nu").arg(user).arg("bash").arg("-c");
                com
            }
            Context::Remote {
                host,
                user,
                identity,
            } => {
                let mut com = process::Command::new("ssh");
                com.arg("-i")
                    .arg(identity)
                    .arg(format!("{}@{}", user, host));
                com
            }
        };

        match pre {
            Some(child) => {
                let stdout = child
                    .stdout
                    .take()
                    .ok_or(anyhow!("error getting output of preceding command"))?;
                com.stdin(stdout);
            }
            None => {}
        }

        com.stdout(process::Stdio::piped())
            .arg(command)
            .spawn()
            .context("error executing command")
    }
}

impl Command for CommandSystem {
    fn run(&mut self, command: &str, context: &Context) -> Result<String> {
        self.run_piped(&vec![(command, context)])
    }

    fn run_piped(&mut self, commands: &Vec<(&str, &Context)>) -> Result<String> {
        self.run_piped(commands)
    }
}

pub struct CommandMock {
    pub commands: Vec<(String, Context)>,
    pub responses: Vec<String>,
}

impl Command for CommandMock {
    fn run(&mut self, command: &str, context: &Context) -> Result<String> {
        let (command_expected, context_expected) = self
            .commands
            .pop()
            .ok_or(anyhow!("no more commands expected"))?;

        assert_eq!(command, command_expected);
        assert_eq!(*context, context_expected);

        Ok(self
            .responses
            .pop()
            .ok_or(anyhow!("no more responses found"))?)
    }

    fn run_piped(&mut self, commands: &Vec<(&str, &Context)>) -> Result<String> {
        let mut resp: Option<String> = None;

        for (command, context) in commands {
            resp = Some(self.run(command, context)?);
        }

        match resp {
            Some(out) => Ok(out),
            None => Err(anyhow!("no output found")),
        }
    }
}

fn check_output(output: &process::Output) -> Result<Vec<u8>> {
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                Ok(output.stdout.clone())
            } else {
                match String::from_utf8(output.stderr.clone()) {
                    Ok(s) => Err(anyhow!(format!(
                        "command finished with status code {}: {}",
                        code, s
                    ))),
                    Err(_) => Err(anyhow!(format!(
                        "command finished with status code {}",
                        code
                    ))),
                }
            }
        }
        None => Err(anyhow!("command was terminated by signal")),
    }
}
