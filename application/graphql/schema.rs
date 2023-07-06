use anyhow::anyhow;
use async_graphql::{Context, EmptySubscription, Object, Result, Schema, ID};
use semver::Version;
use std::str::FromStr;

use crate::auth::{generate_new_token, AuthenticatedUser};
use crate::graphql::types::{
    CrateSummary, CrateVersion, DeletedToken, GeneratedToken, Token, User,
};
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

    #[graphql(name = "crate")]
    async fn get_crate(&self, ctx: &Context<'_>, name: String) -> Result<CrateSummary> {
        let repository = ctx.data::<DynRepository>()?;
        let crate_summary = repository.get_crate_summary(&name).await?;

        Ok(crate_summary.into())
    }

    async fn crate_version(
        &self,
        ctx: &Context<'_>,
        name: String,
        version: Option<String>,
    ) -> Result<CrateVersion> {
        let repository = ctx.data::<DynRepository>()?;

        let version = match version {
            None => {
                let summary = repository.get_crate_summary(&name).await?;
                summary.max_version
            }
            Some(v) => Version::from_str(&v)?,
        };
        let metadata = repository.get_crate_metadata(&name, &version).await?;

        Ok(metadata.into())
    }

    async fn my_tokens(&self, ctx: &Context<'_>) -> Result<Vec<Token>> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>()?;

        let token_items = repository.list_auth_tokens(user.id).await?;
        Ok(token_items.into_iter().map(From::from).collect())
    }

    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<User>> {
        let repository = ctx.data::<DynRepository>()?;
        let user = repository.get_user_by_id(id.parse::<u32>()?).await?;

        Ok(user.map(|u| u.into()))
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let repository = ctx.data::<DynRepository>()?;
        repository
            .get_users()
            .await
            .map(|users| users.into_iter().map(|u| u.into()).collect())
            .map_err(|err| err.into())
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
