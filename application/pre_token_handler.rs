use anyhow::anyhow;
use aws_sdk_dynamodb::Client;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};
use tracing::{error, info, Level};

use raktar::repository::{DynamoDBRepository, Repository};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .json()
        .init();
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (mut event, _context) = event.into_parts();

    // TODO: these should be cached between invocations
    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);
    let repository = DynamoDBRepository::new(db_client);

    info!("pre-token triggered: {}", event);
    match serde_json::from_value::<TriggerEvent>(event.clone()) {
        Ok(trigger_event) => {
            let identities_string = trigger_event.request.user_attributes.identities;
            match serde_json::from_str::<Vec<Identity>>(&identities_string) {
                Ok(identities) => match identities.get(0) {
                    Some(identity) => {
                        let login = &identity.user_id;
                        match repository.get_or_create_user(login).await {
                            Ok(user) => {
                                info!(login, id = user.id, "adding extra claims for user");
                                let response = Response::new(user.id);
                                event
                                    .as_object_mut()
                                    .expect("the trigger event to be an object")
                                    .insert("response".to_string(), to_value(response).unwrap());
                            }
                            Err(err) => {
                                error!("failed to get user: {}", err);
                                return Err(anyhow!("failed to get extra claims for user").into());
                            }
                        };
                    }
                    None => {
                        error!("missing identity in trigger event");
                        return Err(anyhow!("failed to get extra claims for user").into());
                    }
                },
                Err(_) => {
                    error!("identities could not be parsed");
                    return Err(anyhow!("failed to get extra claims for user").into());
                }
            }
        }
        Err(err) => {
            let error_message = err.to_string();
            error!(error_message, "failed to parse trigger event");
            return Err(anyhow!("failed to get extra claims for user").into());
        }
    }

    Ok(event)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    claims_override_details: ClaimsOverrideDetails,
}

impl Response {
    fn new(autogen_id: u32) -> Self {
        Self {
            claims_override_details: ClaimsOverrideDetails {
                claims_to_add_or_override: Claims { autogen_id },
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimsOverrideDetails {
    claims_to_add_or_override: Claims,
}

#[derive(Clone, Debug, Serialize)]
struct Claims {
    autogen_id: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct TriggerEvent {
    request: Request,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    user_attributes: UserAttributes,
}

#[derive(Clone, Debug, Deserialize)]
struct UserAttributes {
    identities: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Identity {
    user_id: String,
}
