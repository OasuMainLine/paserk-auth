use shared::paserk::PaserkClient;
use tokio::sync::OnceCell;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub paserk_client: OnceCell<PaserkClient>,
}
