mod krate;
mod token;
mod user;

use std::sync::Arc;

pub use crate::repository::base::krate::CrateRepository;
pub use crate::repository::base::token::TokenRepository;
pub use crate::repository::base::user::UserRepository;

#[async_trait::async_trait]
pub trait Repository: CrateRepository + UserRepository + TokenRepository {}

pub type DynRepository = Arc<dyn Repository + Send + Sync>;
