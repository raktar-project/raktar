mod base;
pub mod dynamodb;

pub use base::{DynRepository, Repository, UserRepository};
pub use dynamodb::DynamoDBRepository;
