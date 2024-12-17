// (C) Copyright IBM Corp. 2024.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::client::cache::ConfigurationSnapshot;
pub use crate::client::feature_proxy::FeatureProxy;
use crate::client::feature_snapshot::FeatureSnapshot;
use crate::client::http;
pub use crate::client::property_proxy::PropertyProxy;
use crate::client::property_snapshot::PropertySnapshot;
use crate::errors::{ConfigurationAccessError, Error, Result};
use crate::models::Segment;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message;
use tungstenite::WebSocket;

use super::AppConfigurationClient;

/// Defines the IDs of the environment and collection from the IBM Cloud AppConfiguration service.
#[derive(Debug)]
pub struct IBMCloudContext {
    pub(crate) environment_id: String,
    pub(crate) collection_id: String,
}

impl IBMCloudContext {
    /// Creates a new instance with the given environment and collection
    pub fn new(environment_id: &str, collection_id: &str) -> Self {
        Self {
            environment_id: environment_id.to_string(),
            collection_id: collection_id.to_string(),
        }
    }
}

/// AppConfiguration client connection to IBM Cloud.
#[derive(Debug)]
pub struct AppConfigurationClientIBMCloud {
    pub(crate) latest_config_snapshot: Arc<Mutex<ConfigurationSnapshot>>,
    pub(crate) _thread_terminator: std::sync::mpsc::Sender<()>,
}

impl AppConfigurationClientIBMCloud {

    /// Creates a new [`AppConfigurationClient`] connecting to IBM Cloud
    ///
    /// # Arguments
    ///
    /// * `apikey` - The encrypted API key.
    /// * `region` - Region name where the App Configuration service instance is created
    /// * `guid` - Instance ID of the App Configuration service. Obtain it from the service credentials section of the App Configuration dashboard
    /// * `context` - Contains information about the environment and collection to sue.
    /// * `environment_id` - ID of the environment created in App Configuration service instance under the Environments section.
    /// * `collection_id` - ID of the collection created in App Configuration service instance under the Collections section
    pub fn new(apikey: &str, region: &str, guid: &str, context: IBMCloudContext) -> Result<Self> {
        let access_token = http::get_access_token(apikey)?;

        // Populate initial configuration
        let latest_config_snapshot: Arc<Mutex<ConfigurationSnapshot>> = Arc::new(Mutex::new(
            Self::get_configuration_snapshot(&access_token, region, guid, &context)?,
        ));

        // start monitoring configuration
        let terminator = Self::update_cache_in_background(
            latest_config_snapshot.clone(),
            apikey,
            region,
            guid,
            context,
        )?;

        let client = AppConfigurationClientIBMCloud {
            latest_config_snapshot,
            _thread_terminator: terminator,
        };

        Ok(client)
    }

    fn get_configuration_snapshot(
        access_token: &str,
        region: &str,
        guid: &str,
        context: &IBMCloudContext,
    ) -> Result<ConfigurationSnapshot> {
        let configuration = http::get_configuration(
            // TODO: access_token might expire. This will cause issues with long-running apps
            access_token,
            region,
            guid,
            &context.collection_id,
            &context.environment_id,
        )?;
        ConfigurationSnapshot::new(&context.environment_id, configuration)
    }

    fn wait_for_configuration_update(
        socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
        access_token: &str,
        region: &str,
        guid: &str,
        context: &IBMCloudContext,
    ) -> Result<ConfigurationSnapshot> {
        loop {
            // read() blocks until something happens.
            match socket.read()? {
                Message::Text(text) => match text.as_str() {
                    "test message" => {} // periodically sent by the server
                    _ => {
                        return Self::get_configuration_snapshot(
                            access_token,
                            region,
                            guid,
                            context,
                        );
                    }
                },
                Message::Close(_) => {
                    return Err(Error::Other("Connection closed by the server".into()));
                }
                _ => {}
            }
        }
    }

    fn update_configuration_on_change(
        mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
        latest_config_snapshot: Arc<Mutex<ConfigurationSnapshot>>,
        access_token: String,
        region: String,
        guid: String,
        context: IBMCloudContext,
    ) -> std::sync::mpsc::Sender<()> {
        let (sender, receiver) = std::sync::mpsc::channel();

        thread::spawn(move || {
            loop {
                // If the sender has gone (AppConfiguration instance is dropped), then finish this thread
                if let Err(e) = receiver.try_recv() {
                    if e == std::sync::mpsc::TryRecvError::Disconnected {
                        break;
                    }
                }

                let config_snapshot = Self::wait_for_configuration_update(
                    &mut socket,
                    &access_token,
                    &region,
                    &guid,
                    &context,
                );

                match config_snapshot {
                    Ok(config_snapshot) => *latest_config_snapshot.lock()? = config_snapshot,
                    Err(e) => {
                        println!("Waiting for configuration update failed. Stopping to monitor for changes.: {e}");
                        break;
                    }
                }
            }
            Ok::<(), Error>(())
        });

        sender
    }

    fn update_cache_in_background(
        latest_config_snapshot: Arc<Mutex<ConfigurationSnapshot>>,
        apikey: &str,
        region: &str,
        guid: &str,
        context: IBMCloudContext,
    ) -> Result<std::sync::mpsc::Sender<()>> {
        let access_token = http::get_access_token(&apikey)?;
        let (socket, _response) = http::get_configuration_monitoring_websocket(
            &access_token,
            &region,
            &guid,
            &context.collection_id,
            &context.environment_id,
        )?;

        let sender = Self::update_configuration_on_change(
            socket,
            latest_config_snapshot,
            access_token,
            region.to_string(),
            guid.to_string(),
            context,
        );

        Ok(sender)
    }
}

impl AppConfigurationClient for AppConfigurationClientIBMCloud {
    fn get_feature_ids(&self) -> Result<Vec<String>> {
        Ok(self
            .latest_config_snapshot
            .lock()?
            .features
            .keys()
            .cloned()
            .collect())
    }

    fn get_feature(&self, feature_id: &str) -> Result<FeatureSnapshot> {
        let config_snapshot = self.latest_config_snapshot.lock()?;

        // Get the feature from the snapshot
        let feature = config_snapshot.get_feature(feature_id)?;

        // Get the segment rules that apply to this feature
        let segments = {
            let all_segment_ids = feature
                .segment_rules
                .iter()
                .flat_map(|targeting_rule| {
                    targeting_rule
                        .rules
                        .iter()
                        .flat_map(|segment| &segment.segments)
                })
                .cloned()
                .collect::<HashSet<String>>();
            let segments: HashMap<String, Segment> = config_snapshot
                .segments
                .iter()
                .filter(|&(key, _)| all_segment_ids.contains(key))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Integrity DB check: all segment_ids should be available in the snapshot
            if all_segment_ids.len() != segments.len() {
                return Err(ConfigurationAccessError::MissingSegments {
                    resource_id: feature_id.to_string(),
                }
                .into());
            }

            segments
        };

        Ok(FeatureSnapshot::new(feature.clone(), segments))
    }

    fn get_feature_proxy<'a>(&'a self, feature_id: &str) -> Result<FeatureProxy<'a>> {
        // FIXME: there is and was no validation happening if the feature exists.
        // Comments and error messages in FeatureProxy suggest that this should happen here.
        // same applies for properties.
        Ok(FeatureProxy::new(self, feature_id.to_string()))
    }

    fn get_property_ids(&self) -> Result<Vec<String>> {
        Ok(self
            .latest_config_snapshot
            .lock()
            .map_err(|_| ConfigurationAccessError::LockAcquisitionError)?
            .properties
            .keys()
            .cloned()
            .collect())
    }

    fn get_property(&self, property_id: &str) -> Result<PropertySnapshot> {
        let config_snapshot = self.latest_config_snapshot.lock()?;

        // Get the property from the snapshot
        let property = config_snapshot.get_property(property_id)?;

        // Get the segment rules that apply to this property
        let segments = {
            let all_segment_ids = property
                .segment_rules
                .iter()
                .flat_map(|targeting_rule| {
                    targeting_rule
                        .rules
                        .iter()
                        .flat_map(|segment| &segment.segments)
                })
                .cloned()
                .collect::<HashSet<String>>();
            let segments: HashMap<String, Segment> = config_snapshot
                .segments
                .iter()
                .filter(|&(key, _)| all_segment_ids.contains(key))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // Integrity DB check: all segment_ids should be available in the snapshot
            if all_segment_ids.len() != segments.len() {
                // FIXME: Return some kind of DBIntegrity error
                return Err(ConfigurationAccessError::MissingSegments {
                    resource_id: property_id.to_string(),
                }
                .into());
            }

            segments
        };

        Ok(PropertySnapshot::new(property.clone(), segments))
    }

    fn get_property_proxy(&self, property_id: &str) -> Result<PropertyProxy> {
        Ok(PropertyProxy::new(self, property_id.to_string()))
    }
}
