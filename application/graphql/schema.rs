use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema, SimpleObject,
};

use crate::error::internal_error;
use crate::models::crate_details::CrateDetails;
use crate::repository::DynRepository;

pub struct QueryRoot;

#[derive(SimpleObject)]
struct Crate {
    name: String,
}

impl From<CrateDetails> for Crate {
    fn from(value: CrateDetails) -> Self {
        Self { name: value.name }
    }
}

#[Object]
impl QueryRoot {
    async fn crates(&self, ctx: &Context<'_>) -> Result<Vec<Crate>> {
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;
        let crates = repository
            .get_all_crate_details()
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        Ok(crates)
    }
}

pub type RaktarSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(repository: DynRepository) -> RaktarSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(repository)
        .finish()
}
