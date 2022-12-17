use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("command error")]
    Command,
    #[error("error parsing mount information: {0}")]
    MountParsing(String),
    #[error(transparent)]
    Exec(#[from] exec_rs::ExecError),
    #[error("error parsing subvolume: {0}")]
    SubvolumeParsing(String),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error("error parsing subvolume information: {0}")]
    SubvolumeInfoParsing(String),
    #[error("error snapshotting subvolume: {0}")]
    SnapshotSubvolume(String),
    #[error("error deleting subvolume: {0}")]
    DeleteSubvolume(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("error parsing duration")]
    DurationConversionError,
    #[error("error converting btrfs path into filesystem path")]
    PathConversionError,
    #[error("error creating snapshot: {0}")]
    SnapshotCreation(String),
}
