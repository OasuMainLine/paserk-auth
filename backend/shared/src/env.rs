pub fn load_env() -> Result<(), dotenvy::Error> {
    dotenvy::from_filename_override(".env.local")?;
    dotenvy::from_filename_override(".env")?;

    Ok(())
}
