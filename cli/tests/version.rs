use std::process::Command;

const VERSION: &str = env!("VULCANUM_VERSION");

#[test]
fn version_flags_print_the_embedded_version() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_vulcanum"))
            .arg(flag)
            .output()
            .expect("version command should run");

        assert!(output.status.success(), "{flag} should exit successfully");
        assert_eq!(
            String::from_utf8(output.stdout)
                .expect("version output should be UTF-8")
                .trim(),
            format!("vulcanum {VERSION}")
        );
        assert!(output.stderr.is_empty(), "{flag} should not write stderr");
    }
}
