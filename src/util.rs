/// Question mark operator for Option<Result<T, E>>
/// inner_bail!( Option<Result<T, E>> ) -> Option<T>
#[macro_export]
macro_rules! inner_bail {
    ($e:expr) => {
        match $e {
            None => None,
            Some(Ok(x)) => Some(x),
            Some(Err(e)) => return Err(e.into()),
        }
    };
}
