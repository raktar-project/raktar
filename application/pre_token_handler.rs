use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;
use serde_json::Value;
use tracing::{error, info, Level};

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
    let (event, _context) = event.into_parts();
    info!("pre-token triggered: {}", event);
    match serde_json::from_value::<TriggerEvent>(event.clone()) {
        Ok(trigger_event) => {
            let identities_string = trigger_event.request.user_attributes.identities;
            match serde_json::from_str::<Vec<Identity>>(&identities_string) {
                Ok(identities) => match identities.get(0) {
                    Some(identity) => {
                        let user_id = &identity.user_id;
                        info!(user_id, "adding extra claims for user");
                    }
                    None => {
                        error!("missing identity in trigger event");
                    }
                },
                Err(_) => {
                    error!("identities could not be parsed");
                }
            }
        }
        Err(err) => {
            let error_message = err.to_string();
            error!(error_message, "failed to parse trigger event");
        }
    }

    Ok(event)
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
