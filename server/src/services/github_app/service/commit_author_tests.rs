use crate::services::github_app::service::commit_author::commit_author_for;

#[test]
fn commit_author_uses_github_bot_noreply_identity() {
    let author = commit_author_for(12_345, "vulcanum-app[bot]");

    assert_eq!(author.name, "vulcanum-app[bot]");
    assert_eq!(
        author.email,
        "12345+vulcanum-app[bot]@users.noreply.github.com"
    );
}
