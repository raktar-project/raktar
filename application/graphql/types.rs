use async_graphql::SimpleObject;
use raktar::models::crate_details::CrateDetails;
use raktar::models::token::TokenItem;

#[derive(SimpleObject)]
pub struct Crate {
    name: String,
}

impl From<CrateDetails> for Crate {
    fn from(value: CrateDetails) -> Self {
        Self { name: value.name }
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
