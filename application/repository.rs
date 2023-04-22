mod base;
pub mod dynamodb;

pub use base::{DynRepository, Repository};
pub use dynamodb::DynamoDBRepository;
