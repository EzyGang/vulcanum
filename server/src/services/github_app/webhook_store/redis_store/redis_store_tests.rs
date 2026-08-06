use redis::ToRedisArgs;

use crate::services::github_app::webhook_store::redis_store::optional_i64_arg;

#[test]
fn optional_comment_id_preserves_script_argument_position() {
    let absent = optional_i64_arg(None).to_redis_args();
    let present = optional_i64_arg(Some(42)).to_redis_args();

    assert_eq!(absent, vec![Vec::<u8>::new()]);
    assert_eq!(present, vec![b"42".to_vec()]);
}
