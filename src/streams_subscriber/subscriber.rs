//!
//! Channel Subscriber
//!
use crate::streams_subscriber::random_seed;

use iota_streams::app::transport::tangle::client::{{Client,SendOptions},iota_client::Client as OtherClient};
use iota_streams::app_channels::api::tangle::{Address, Subscriber};
use iota_streams::app::transport::tangle::{ TangleAddress};
use futures::executor::block_on;
use base64::{decode_config, URL_SAFE_NO_PAD};
use std::str::FromStr;
use anyhow::Result;

///
/// Channel subscriber
///
pub struct Channel {
    subscriber: Subscriber<Client>,
    announcement_link: Address,
    subscription_link: Address,
    channel_address: String,
}

impl Channel {
    ///
    /// Initialize the subscriber
    ///
    pub fn new(
        node: &str,
        channel_address: &str,
        announcement_tag: String,
        seed_option: Option<String>,
    ) -> Channel {
        let seed = match seed_option {
            Some(seed) => seed,
            None => random_seed(),
        };

        let send_options: SendOptions = SendOptions {
          url: node.to_string(),
          local_pow: false,
        };
      
        let iota_client = block_on(
          OtherClient::builder()
            .with_node(&node)
            .unwrap()
            .with_local_pow(false)
            .finish(),
        )
        .unwrap();
        let client = Client::new(send_options, iota_client);
        // let client: Client = Client::new_from_url(&node);
        let subscriber = Subscriber::new(&seed, client);

        Self {
            subscriber: subscriber,
            announcement_link: TangleAddress::from_str(&(channel_address.to_string()+ &":".to_string() + &announcement_tag)).unwrap(),
            subscription_link: Address::default(),
            channel_address: channel_address.to_string(),
        }
    }

    ///
    /// Connect
    ///
    pub async fn connect(&mut self) -> Result<String> {
        self.subscriber
            .receive_announcement(&self.announcement_link).await.unwrap();
        Ok(self.subscription_link.msgid.to_string())
    }
    ///
    /// Read signed packet
    ///
    pub async fn read_signed(
        &mut self,
        signed_packet_tag: String,
    ) -> Result<Vec<(Option<String>, Option<String>)>> {
        let mut response: Vec<(Option<String>, Option<String>)> = Vec::new();

        let link = TangleAddress::from_str(&(self.channel_address.to_string()+ &":".to_string() + &signed_packet_tag)).unwrap();
        println!("link: {}",link);
        let (_, _, masked_payload) = self.subscriber.receive_signed_packet(&link).await.unwrap();
        response.push((
            None,
            unwrap_data(&String::from_utf8(masked_payload.0.to_vec()).unwrap()).unwrap(), //Iot2Tanagle currently only support public masseges
        ));

        Ok(response)
    }

    ///
    /// Generates the next message in the channels
    ///
    pub async fn get_next_message(&mut self) -> Option<Vec<String>> {
        let mut ids: Vec<String> = vec![];

        let mut msgs = self.subscriber.fetch_next_msgs().await;

        for msg in &msgs {
            ids.push(msg.link.msgid.to_string());
        }

        while !msgs.is_empty() {
            msgs = self.subscriber.fetch_next_msgs().await;

            for msg in &msgs {
                ids.push(msg.link.msgid.to_string());
            }
        }

        Some(ids)
    }
}

pub fn unwrap_data(data: &str) -> failure::Fallible<Option<String>> {
    let data_str = data.to_string();
    if data_str.len() == 0 {
        return Ok(None);
    }
    let raw = &data.to_string();
    let decode_data = decode_config(&raw, URL_SAFE_NO_PAD)?;
    Ok(Some(String::from_utf8(decode_data).unwrap()))
}
