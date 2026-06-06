use std::process::Stdio;

pub(crate) fn remove_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let _ = std::process::Command::new("docker")
        .args(["rm", "-f", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
