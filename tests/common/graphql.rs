use async_graphql::Request;
use raktar::auth::AuthenticatedUser;

#[allow(dead_code)] // not all tests use this
pub fn build_request(request_str: &str, user_id: u32) -> Request {
    let authenticated_user = AuthenticatedUser { id: user_id };
    let request: Request = request_str.into();
    request.data(authenticated_user)
}
