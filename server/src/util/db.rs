pub fn ensure_rows_affected<E>(rows: u64, not_found: E) -> Result<(), E> {
    match rows {
        0 => Err(not_found),
        _ => Ok(()),
    }
}
