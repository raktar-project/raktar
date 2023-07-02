use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use futures::future::try_join_all;
use semver::Version;

use crate::models::crate_summary::CrateSummary as CrateSummaryModel;
use crate::models::metadata::Metadata;
use crate::models::token::TokenItem;
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
}

impl From<UserModel> for User {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id.into(),
            login: value.login,
        }
    }
}

#[derive(SimpleObject)]
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
    all_versions: Vec<String>,
}

impl CrateVersion {
    pub fn new(metadata: Metadata, versions: Vec<Version>) -> Self {
        let all_versions = versions.into_iter().map(|v| v.to_string()).collect();
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
            all_versions,
        }
    }
}

#[derive(SimpleObject)]
pub struct Token {
    pub id: ID,
    user_id: u32,
    name: String,
}

impl From<TokenItem> for Token {
    fn from(item: TokenItem) -> Self {
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
