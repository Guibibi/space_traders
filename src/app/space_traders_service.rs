use tokio::sync::mpsc::{channel, Sender};

use space_traders_api::{
    apis::configuration::Configuration,
    models::{Agent, GetMyAgent200Response},
};
use tokio::runtime::{self, Runtime};

use super::Messages;

// Implement the SpaceTradersService struct
pub struct SpaceTradersService {
    client: Configuration,
    rt: Runtime,
    sender: Sender<Messages>,
}

impl SpaceTradersService {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let user_token =
            std::env::var("ACCOUNT_TOKEN").expect("ACCOUNT_TOKEN environement variable not set.");
        let mut client: Configuration = Configuration::new();
        client.bearer_access_token = Some(user_token);

        let (sender, _) = channel(1);
        Self {
            client,
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime"),
            sender,
        }
    }

    pub async fn get_agent(&self) -> Result<Agent, Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let request = space_traders_api::apis::agents_api::get_my_agent(&client).await;
        Ok(*request.unwrap().data)
    }
}
