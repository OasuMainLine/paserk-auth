use std::path::Path;

pub fn from_filename_override_optional<P>(filename: P) -> Result<Option<()>, dotenvy::Error>
where
    P: AsRef<Path>,
{
    let result = dotenvy::from_filename_override(filename);
    if let Err(error) = result {
        if error.not_found() {
            return Ok(None);
        }
        return Err(error);
    }
    return Ok(Some(()));
}

pub fn load_env() -> Result<(), dotenvy::Error> {
    dotenvy::from_filename_override(".env.local")?;
    from_filename_override_optional(".env")?;

    Ok(())
}
