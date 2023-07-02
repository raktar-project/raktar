use anyhow::anyhow;
use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use semver::Version;
use std::str::FromStr;
use tokio::join;

use crate::auth::{generate_new_token, AuthenticatedUser};
use crate::graphql::types::{Crate, CrateSummary, DeletedToken, GeneratedToken, Token};
use crate::repository::DynRepository;

pub struct Query;

#[Object]
impl Query {
    async fn crates(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<CrateSummary>> {
        let repository = ctx.data::<DynRepository>()?;

        let limit = limit.unwrap_or(10);
        if limit > 20 {
            return Err(anyhow!(format!("limit must be less than {}", 20)).into());
        }
        let crates = repository
            .get_all_crate_details(filter, limit)
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        Ok(crates)
    }

    async fn crate_version(
        &self,
        ctx: &Context<'_>,
        name: String,
        version: Option<String>,
    ) -> Result<Crate> {
        let repository = ctx.data::<DynRepository>()?;

        let summary = repository.get_crate_summary(&name).await?;
        let version = version.map_or(Ok(summary.max_version), |v| Version::from_str(&v))?;

        let metadata_fut = repository.get_crate_metadata(&name, &version);
        let versions_fut = repository.list_crate_versions(&name);

        let (metadata_result, versions_result) = join!(metadata_fut, versions_fut);
        let metadata = metadata_result?;
        let versions = versions_result?;

        Ok(Crate::new(metadata, versions, summary.owners))
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
        let token_item = repository
            .store_auth_token(key.as_bytes(), name, user.id)
            .await?;
        let token: Token = token_item.into();
        let generated_token = GeneratedToken {
            id: token.id.clone(),
            token,
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
