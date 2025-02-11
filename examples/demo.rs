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

use appconfiguration::{
    AppConfigurationClient, AppConfigurationClientIBMCloud, ConfigurationId, Entity, Feature, Value,
};

use dotenvy::dotenv;

use spinners_rs::{Spinner, Spinners};
use std::error::Error;
use std::{collections::HashMap, env, thread, time::Duration};

#[derive(Debug)]
struct CustomerEntity {
    id: String,
    name: String,
    city: String,
}

impl Entity for CustomerEntity {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_attributes(&self) -> HashMap<String, Value> {
        HashMap::from_iter(vec![("city".to_string(), Value::from(self.city.clone()))])
    }
}

impl std::fmt::Display for CustomerEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.city)
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    dotenv().ok();
    let region = env::var("REGION").expect("REGION should be set.");
    let guid = env::var("GUID").expect("GUID should be set.");
    let apikey = env::var("APIKEY").expect("APIKEY should be set.");
    let collection_id = env::var("COLLECTION_ID").expect("COLLECTION_ID should be set.");
    let environment_id = env::var("ENVIRONMENT_ID").expect("ENVIRONMENT_ID should be set.");

    let configuration = ConfigurationId::new(guid, environment_id, collection_id);
    let client = AppConfigurationClientIBMCloud::new(&apikey, &region, configuration)?;

    let e1 = CustomerEntity {
        id: "jerry".to_string(),
        name: "Jerry".to_string(),
        city: "Yorktown".to_string(),
    };

    let e2 = CustomerEntity {
        id: "asalva".to_string(),
        name: "Salva".to_string(),
        city: "Yorktown".to_string(),
    };

    let e3 = CustomerEntity {
        id: "javier".to_string(),
        name: "Javi".to_string(),
        city: "Madrid".to_string(),
    };

    let e4 = CustomerEntity {
        id: "rainer".to_string(),
        name: "Rainer".to_string(),
        city: "Stuttgart".to_string(),
    };

    let feature_id = "discount";
    let feature = client.get_feature_proxy(feature_id)?;

    println!("\n\nCurrent discounts\n");

    println!("    City | {:^13} | {:6} | {:9}", e1.city, e3.city, e4.city);
    println!(
        "    Name | {:>5} | {:>5} | {:>6} | {:>9}\n",
        e1.name, e2.name, e3.name, e4.name
    );
    let mut row: String = "".to_string();

    let mut sp: Spinner = Spinners::Dots.into();
    sp.start();
    loop {
        let value_james: i64 = feature.get_value(&e1)?.try_into()?;
        let value_mary: i64 = feature.get_value(&e2)?.try_into()?;
        let value_mateo: i64 = feature.get_value(&e3)?.try_into()?;
        let value_sofia: i64 = feature.get_value(&e4)?.try_into()?;

        let new_row = format!(
            "{:>5} | {:>5} | {:>6} | {:>9}",
            value_james, value_mary, value_mateo, value_sofia
        );
        if new_row != row {
            row = new_row;
            sp.stop_with_message(format!(
                "{} | {}\n",
                chrono::Local::now().format("%H:%M:%S"),
                row
            ));
            sp.start();
        }

        thread::sleep(Duration::from_secs(2));
    }
}
