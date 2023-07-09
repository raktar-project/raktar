use crate::error::AppError;
use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use futures::future::try_join_all;

use crate::models::crate_summary::CrateSummary as CrateSummaryModel;
use crate::models::metadata::Metadata;
use crate::models::token::Token as TokenModel;
use crate::models::user::User as UserModel;
use crate::repository::DynRepository;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CrateSummary {
    id: ID,
    name: String,
    max_version: String,
    description: String,
    #[graphql(skip)]
    owner_ids: Vec<u32>,
}

#[ComplexObject]
impl CrateSummary {
    async fn owners(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let repository = ctx.data::<DynRepository>()?;

        let queries: Vec<_> = self
            .owner_ids
            .iter()
            .map(|id| repository.get_user_by_id(*id))
            .collect();

        let res = try_join_all(queries).await?;
        let users: Vec<_> = res.into_iter().flatten().map(|u| u.into()).collect();

        Ok(users)
    }

    async fn versions(&self, ctx: &Context<'_>) -> Result<Vec<String>> {
        let repository = ctx.data::<DynRepository>()?;

        let versions = repository
            .list_crate_versions(&self.name)
            .await?
            .into_iter()
            .map(|v| v.to_string())
            .collect();

        Ok(versions)
    }
}

impl From<CrateSummaryModel> for CrateSummary {
    fn from(value: CrateSummaryModel) -> Self {
        Self {
            id: value.name.clone().into(),
            name: value.name,
            max_version: value.max_version.to_string(),
            description: value.description,
            owner_ids: value.owners,
        }
    }
}

#[derive(SimpleObject)]
pub struct User {
    id: ID,
    login: String,
    given_name: String,
    family_name: String,
}

impl From<UserModel> for User {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id.into(),
            login: value.login,
            given_name: value.given_name,
            family_name: value.family_name,
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct CrateVersion {
    id: ID,
    name: String,
    version: String,
    authors: Vec<String>,
    description: Option<String>,
    readme: Option<String>,
    keywords: Vec<String>,
    categories: Vec<String>,
    repository: Option<String>,
}

impl From<Metadata> for CrateVersion {
    fn from(metadata: Metadata) -> Self {
        Self {
            id: format!("{}-{}", &metadata.name, &metadata.vers).into(),
            name: metadata.name,
            version: metadata.vers.to_string(),
            authors: metadata.authors,
            description: metadata.description,
            readme: metadata.readme,
            keywords: metadata.keywords,
            categories: metadata.categories,
            repository: metadata.repository.map(From::from),
        }
    }
}

#[ComplexObject]
impl CrateVersion {
    #[graphql(name = "crate")]
    async fn get_crate(&self, ctx: &Context<'_>) -> Result<CrateSummary> {
        let repository = ctx.data::<DynRepository>()?;
        if let Some(crate_summary) = repository.get_crate_summary(&self.name).await? {
            Ok(crate_summary.into())
        } else {
            Err(AppError::NonExistentCrate(self.name.clone()).into())
        }
    }
}

#[derive(SimpleObject)]
pub struct Token {
    pub id: ID,
    user_id: u32,
    name: String,
}

impl From<TokenModel> for Token {
    fn from(item: TokenModel) -> Self {
        Self {
            id: item.token_id.into(),
            user_id: item.user_id,
            name: item.name,
        }
    }
}

#[derive(SimpleObject)]
pub struct GeneratedToken {
    pub id: ID,
    pub key: String,
    pub token: Token,
}

#[derive(SimpleObject)]
pub struct DeletedToken {
    pub id: String,
}
