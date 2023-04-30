use async_graphql::{Context, EmptySubscription, Object, Result, Schema, SimpleObject};

use crate::graphql::handler::AuthenticatedUser;
use raktar::auth::generate_new_token;
use raktar::error::internal_error;
use raktar::models::crate_details::CrateDetails;
use raktar::repository::DynRepository;

pub struct Query;

#[derive(SimpleObject)]
struct Crate {
    name: String,
}

#[derive(SimpleObject)]
struct GeneratedToken {
    token: String,
}

#[derive(SimpleObject)]
struct Token {
    user_id: u32,
    name: String,
}

impl From<CrateDetails> for Crate {
    fn from(value: CrateDetails) -> Self {
        Self { name: value.name }
    }
}

#[Object]
impl Query {
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

    async fn my_tokens(&self, ctx: &Context<'_>) -> Result<Vec<Token>> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        let token_items = repository.list_auth_tokens(user.id).await?;
        Ok(token_items
            .into_iter()
            .map(|item| Token {
                user_id: item.user_id,
                name: item.name,
            })
            .collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn generate_token(&self, ctx: &Context<'_>, name: String) -> Result<GeneratedToken> {
        let user = ctx.data::<AuthenticatedUser>()?;
        let repository = ctx.data::<DynRepository>().map_err(|_| internal_error())?;

        let generated = generate_new_token();
        repository
            .store_auth_token(&generated.secure_hash, name, user.id)
            .await?;
        let token = GeneratedToken {
            token: generated.plaintext,
        };

        Ok(token)
    }
}

pub type RaktarSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(repository: DynRepository) -> RaktarSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(repository)
        .finish()
}
