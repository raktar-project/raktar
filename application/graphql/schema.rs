use async_graphql::{Context, EmptySubscription, Object, Result, Schema};

use crate::graphql::handler::AuthenticatedUser;
use crate::graphql::types::{Crate, CrateSummary, DeletedToken, GeneratedToken, Token};
use raktar::auth::generate_new_token;
use raktar::error::internal_error;
use raktar::repository::DynRepository;

pub struct Query;

#[Object]
impl Query {
    async fn crates(&self, ctx: &Context<'_>) -> Result<Vec<CrateSummary>> {
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;
        let crates = repository
            .get_all_crate_details()
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        Ok(crates)
    }

    async fn crate_details(&self, ctx: &Context<'_>, name: String) -> Result<Crate> {
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        let details = repository.get_crate_details(&name).await?;
        let metadata = repository
            .get_crate_metadata(&name, &details.max_version)
            .await?;

        Ok(metadata.into())
    }

    async fn my_tokens(&self, ctx: &Context<'_>) -> Result<Vec<Token>> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        let token_items = repository.list_auth_tokens(user.id).await?;
        Ok(token_items.into_iter().map(From::from).collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn generate_token(&self, ctx: &Context<'_>, name: String) -> Result<GeneratedToken> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        let key = generate_new_token();
        let token = repository
            .store_auth_token(key.as_bytes(), name, user.id)
            .await?;
        let generated_token = GeneratedToken {
            token: token.into(),
            key,
        };

        Ok(generated_token)
    }

    async fn delete_token(&self, ctx: &Context<'_>, token_id: String) -> Result<DeletedToken> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        repository
            .delete_auth_token(user.id, token_id.clone())
            .await?;

        Ok(DeletedToken { id: token_id })
    }
}

pub type RaktarSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(repository: DynRepository) -> RaktarSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(repository)
        .finish()
}
