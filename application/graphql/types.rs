use async_graphql::SimpleObject;
use semver::Version;

use raktar::models::crate_details::CrateDetails;
use raktar::models::metadata::Metadata;
use raktar::models::token::TokenItem;

#[derive(SimpleObject)]
pub struct CrateSummary {
    name: String,
    max_version: String,
    description: String,
}

impl From<CrateDetails> for CrateSummary {
    fn from(details: CrateDetails) -> Self {
        Self {
            name: details.name,
            max_version: details.max_version.to_string(),
            description: details.description,
        }
    }
}

#[derive(SimpleObject)]
pub struct Crate {
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
    id: String,
    user_id: u32,
    name: String,
}

impl From<TokenItem> for Token {
    fn from(item: TokenItem) -> Self {
        Self {
            id: item.token_id,
            user_id: item.user_id,
            name: item.name,
        }
    }
}

#[derive(SimpleObject)]
pub struct GeneratedToken {
    pub(crate) key: String,
    pub(crate) token: Token,
}

#[derive(SimpleObject)]
pub struct DeletedToken {
    pub id: String,
}
