use exec_rs::{Context, Exec};

use crate::backup_error::BackupError;

pub trait CommandReadLink {
    /// Read link
    ///
    /// Checks whether the given path is a link and returns the link and the target of the link.
    /// If the path is not a link, only the path is returned.
    ///
    /// * `path` - path of the potential link
    /// * `exec` - command executor
    /// * `context` - context in which to execute the command
    ///
    fn read_link(&mut self, path: &str, context: &Context) -> Result<Vec<String>, BackupError>;
}

impl<T: Exec> CommandReadLink for super::Commander<T> {
    fn read_link(&mut self, path: &str, context: &Context) -> Result<Vec<String>, BackupError> {
        let command_output = self
            .exec
            .exec("readlink", &["-f", path], Some(context))?
            .trim()
            .to_string();

        let mut result = Vec::new();

        result.push(path.to_string());

        if path != command_output {
            result.push(command_output);
        }

        Ok(result)
    }
}
