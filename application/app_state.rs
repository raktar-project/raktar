use crate::repository::Repository;
use crate::storage::CrateStorage;

#[derive(Clone)]
pub struct AppState<R: Repository, S: CrateStorage> {
    pub repository: R,
    pub storage: S,
}
