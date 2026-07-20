use crate::commands::skills::{InstallerCommand, PackageRunner, Skill};

#[test]
fn installer_commands_target_the_vulcanum_repository() {
    let cases = [
        (PackageRunner::Pnpm, "pnpm", vec!["dlx", "skills"]),
        (PackageRunner::Npx, "npx", vec!["skills"]),
        (PackageRunner::Bunx, "bunx", vec!["skills"]),
        (PackageRunner::Yarn, "yarn", vec!["dlx", "skills"]),
    ];

    for (runner, executable, prefix) in cases {
        let command = InstallerCommand::new(runner, Some(Skill::VulcanumTicketTemplate));
        let mut expected = prefix;
        expected.extend([
            "add",
            "EzyGang/vulcanum",
            "--skill",
            "vulcanum-ticket-template",
        ]);

        assert_eq!(command.executable, executable);
        assert_eq!(command.args, expected);
    }
}

#[test]
fn installer_selects_both_vulcanum_skills_when_name_is_omitted() {
    let command = InstallerCommand::new(PackageRunner::Pnpm, None);

    assert_eq!(
        command.args,
        [
            "dlx",
            "skills",
            "add",
            "EzyGang/vulcanum",
            "--skill",
            "vulcanum-cli",
            "--skill",
            "vulcanum-ticket-template",
        ]
    );
}
