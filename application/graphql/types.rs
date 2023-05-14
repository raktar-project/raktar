use async_graphql::{SimpleObject, ID};
use semver::Version;

use crate::models::crate_details::CrateDetails;
use crate::models::metadata::Metadata;
use crate::models::token::TokenItem;

#[derive(SimpleObject)]
pub struct CrateSummary {
    id: ID,
    name: String,
    max_version: String,
    description: String,
}

impl From<CrateDetails> for CrateSummary {
    fn from(details: CrateDetails) -> Self {
        Self {
            id: details.name.clone().into(),
            name: details.name,
            max_version: details.max_version.to_string(),
            description: details.description,
        }
    }
}

#[derive(SimpleObject)]
pub struct Crate {
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

impl Crate {
    pub(crate) fn new(metadata: Metadata, versions: Vec<Version>) -> Self {
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
