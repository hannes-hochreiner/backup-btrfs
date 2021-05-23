use crate::command::{Command, CommandSystem, Context};

#[test]
fn run() {
    let mut com = CommandSystem {};

    assert_eq!(com.run("ls Cargo.toml", &Context::Local{user: "hannes".into()}).unwrap(), "Cargo.toml\n");
}

#[test]
fn run_piped() {
    let mut com = CommandSystem {};

    assert_eq!(com.run_piped(&vec!(
        ("cat Cargo.toml", &Context::Local{user: "hannes".into()}),
        ("grep name", &Context::Local{user: "hannes".into()}),
    )).unwrap(), "name = \"backup-local-rs\"\n");
}
