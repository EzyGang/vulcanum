#[must_use]
pub fn resolve_update_deref<'a, T, U>(
    update: &'a Option<Option<T>>,
    existing: Option<&'a U>,
) -> Option<&'a U>
where
    T: AsRef<U>,
    U: ?Sized,
{
    match update {
        Some(Some(value)) => Some(value.as_ref()),
        Some(None) => None,
        None => existing,
    }
}

#[must_use]
pub fn resolve_update_option<T>(update: &Option<Option<T>>, existing: Option<T>) -> Option<T>
where
    T: Copy,
{
    match update {
        Some(value) => *value,
        None => existing,
    }
}
