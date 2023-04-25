use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}

pub type RaktarSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> RaktarSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}
