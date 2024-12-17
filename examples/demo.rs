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

use std::{collections::HashMap, env, thread, time::Duration};

use appconfiguration_rust_sdk::{
    AppConfigurationClient, AppConfigurationClientIBMCloud, Entity, Feature, IBMCloudContext,
    Property, Value,
};
use dotenvy::dotenv;
use std::error::Error;

#[derive(Debug)]
struct CustomerEntity {
    id: String,
    city: String,
    radius: u32,
}

impl Entity for CustomerEntity {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_attributes(&self) -> HashMap<String, Value> {
        HashMap::from_iter(vec![
            ("city".to_string(), Value::from(self.city.clone())),
            ("radius".to_string(), Value::from(self.radius as u64)),
        ])
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    dotenv().ok();
    let region = env::var("REGION").expect("REGION should be set.");
    let guid = env::var("GUID").expect("GUID should be set.");
    let apikey = env::var("APIKEY").expect("APIKEY should be set.");
    let collection_id = env::var("COLLECTION_ID").expect("COLLECTION_ID should be set.");
    let environment_id = env::var("ENVIRONMENT_ID").expect("ENVIRONMENT_ID should be set.");
    let feature_id = env::var("FEATURE_ID").expect("FEATURE_ID should be set.");
    let property_id = env::var("PROPERTY_ID").expect("PROPERTY_ID should be set.");

    let context = IBMCloudContext::new(&environment_id, &collection_id);
    let client = AppConfigurationClientIBMCloud::new(
        &apikey,
        &region,
        &guid,
        context,
    )?;

    let entity = CustomerEntity {
        id: "user123".to_string(),
        city: "Bangalore".to_string(),
        radius: 60,
    };

    println!("The information is displayed every 5 seconds.");
    println!("Try changing the configuraiton in the App Configuration instances.");

    loop {
        println!("\n\nFEATURE FLAG OPERATIONS\n");

        match client.get_feature_proxy(&feature_id) {
            Ok(feature) => {
                println!("Feature name: {}", feature.get_name()?);
                let value = feature.get_value(&entity)?;
                println!("Is feature enabled: {}", feature.is_enabled()?);
                println!("Feature evaluated value is: {value:?}");
            }
            Err(error) => {
                println!("There was an error getting the Feature Flag. Error {error}",);
            }
        }

        println!("\n\nPROPERTY OPERATIONS\n");
        match client.get_property_proxy(&property_id) {
            Ok(property) => {
                println!("Property name: {}", property.get_name()?);
                let value = property.get_value(&entity)?;
                println!("Property evaluated value is: {value:?}");
            }
            Err(error) => {
                println!("There was an error getting the Property. Error {error}",);
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}
