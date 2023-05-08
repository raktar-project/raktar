use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use semver::Version;
use std::str::FromStr;

use crate::graphql::handler::AuthenticatedUser;
use crate::graphql::types::{Crate, CrateSummary, DeletedToken, GeneratedToken, Token};
use raktar::auth::generate_new_token;
use raktar::repository::DynRepository;

pub struct Query;

#[Object]
impl Query {
    async fn crates(&self, ctx: &Context<'_>) -> Result<Vec<CrateSummary>> {
        let repository = ctx.data::<DynRepository>()?;
        let crates = repository
            .get_all_crate_details()
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        Ok(crates)
    }

    async fn crate_details(
        &self,
        ctx: &Context<'_>,
        name: String,
        version: Option<String>,
    ) -> Result<Crate> {
        let repository = ctx.data::<DynRepository>()?;

        let version = match version {
            None => {
                let details = repository.get_crate_details(&name).await?;
                details.max_version
            }
            Some(v) => Version::from_str(&v)?,
        };
        // TODO: these two should be in parallel
        let metadata = repository.get_crate_metadata(&name, &version).await?;
        let versions = repository.list_crate_versions(&name).await?;

        let krate = Crate::new(metadata, versions);

        Ok(krate)
    }

    async fn my_tokens(&self, ctx: &Context<'_>) -> Result<Vec<Token>> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>()?;

        let token_items = repository.list_auth_tokens(user.id).await?;
        Ok(token_items.into_iter().map(From::from).collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn generate_token(&self, ctx: &Context<'_>, name: String) -> Result<GeneratedToken> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>()?;

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
        let repository = ctx.data::<DynRepository>()?;

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
