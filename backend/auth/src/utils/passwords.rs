use thiserror::Error;
use zxcvbn::{Score, zxcvbn};

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Weak password detected: {feedback:?}")]
    WeakPasswordError { feedback: Option<String> },
}
pub fn check_password(password: &str) -> Result<(), PasswordError> {
    let estimate = zxcvbn(&password, &[]);

    if estimate.score() >= Score::Three {
        Ok(())
    } else {
        let feedback = estimate
            .feedback()
            .and_then(|f| f.warning())
            .map(|w| w.to_string());
        Err(PasswordError::WeakPasswordError { feedback })
    }
}
