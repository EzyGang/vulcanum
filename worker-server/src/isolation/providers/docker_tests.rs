use uuid::Uuid;

use crate::isolation::providers::docker::{cleanup_docker_workdir, DockerIsolation};
use crate::isolation::providers::kata::KataIsolation;

#[test]
fn container_image_default() {
    let isolation = DockerIsolation::new(Some("test-runtime"), "test-image:v1".to_owned());
    assert!(!isolation.image.is_empty());
}

#[test]
fn docker_plain_no_runtime() {
    let isolation = DockerIsolation::new(None, "test-image:v1".to_owned());
    assert!(isolation.runtime.is_none());
    assert!(!isolation.image.is_empty());
}

#[test]
fn kata_inner_image_default() {
    let isolation = KataIsolation::new("test-image:v1".to_owned());
    assert!(!isolation.inner.image.is_empty());
}

#[test]
fn kata_inner_image_custom() {
    let isolation = KataIsolation::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(isolation.inner.image, "my-registry/agent:v1");
}

#[tokio::test]
async fn docker_cleanup_removes_safe_workdir() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", Uuid::new_v4()));
    let nested = workdir.join("home").join("file.txt");
    tokio::fs::create_dir_all(nested.parent().expect("nested file should have parent"))
        .await
        .expect("workdir should be created");
    tokio::fs::write(&nested, "data")
        .await
        .expect("nested file should be written");

    cleanup_docker_workdir(&workdir, "unused-image", None).await;

    assert!(!workdir.exists());
}

#[tokio::test]
async fn docker_cleanup_refuses_unsafe_workdir() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-other-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&workdir)
        .await
        .expect("workdir should be created");

    cleanup_docker_workdir(&workdir, "unused-image", None).await;

    assert!(workdir.exists());
    let _ = tokio::fs::remove_dir_all(&workdir).await;
}
