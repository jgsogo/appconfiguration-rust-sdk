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

use std::{collections::HashMap, sync::PoisonError};

use crate::models::{Configuration, Feature, Property, Segment};

#[derive(Debug)]
pub enum ConfigurationAccessError {
    LockAcquisitionError,
    EnvironmentNotFound { environment_id: String },
    FeatureNotFound { feature_id: String },
    PropertyNotFound { property_id: String },
    MissingSegments { resource_id: String },
}

impl std::error::Error for ConfigurationAccessError {}

impl std::fmt::Display for ConfigurationAccessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockAcquisitionError => {
                write!(formatter, "Error acquiring index cache lock.")
            }
            Self::EnvironmentNotFound { environment_id } => {
                write!(
                    formatter,
                    "Environment `{}` indicated as key not found in the configuration instance.",
                    environment_id
                )
            }
            Self::FeatureNotFound { feature_id } => {
                write!(formatter, "Feature `{feature_id}` not found.")
            }
            Self::PropertyNotFound { property_id } => {
                write!(formatter, "Property `{property_id}` not found.")
            }
            Self::MissingSegments { resource_id } => {
                write!(formatter, "Missing Segments for Resource `{resource_id}`.")
            }
        }
    }
}

impl<T> From<PoisonError<T>> for ConfigurationAccessError {
    fn from(_value: PoisonError<T>) -> Self {
        ConfigurationAccessError::LockAcquisitionError
    }
}

pub type Result<T> = std::result::Result<T, ConfigurationAccessError>;

#[derive(Debug, Default)]
pub(crate) struct ConfigurationSnapshot {
    pub(crate) features: HashMap<String, Feature>,
    pub(crate) properties: HashMap<String, Property>,
    pub(crate) segments: HashMap<String, Segment>,
}

impl ConfigurationSnapshot {
    pub fn get_feature(&self, feature_id: &str) -> Result<&Feature> {
        self.features
            .get(feature_id)
            .ok_or_else(|| ConfigurationAccessError::FeatureNotFound {
                feature_id: feature_id.to_string(),
            })
    }

    pub fn get_property(&self, property_id: &str) -> Result<&Property> {
        self.properties
            .get(property_id)
            .ok_or_else(|| ConfigurationAccessError::PropertyNotFound {
                property_id: property_id.to_string(),
            })
    }

    pub fn new(environment_id: &str, configuration: Configuration) -> Result<Self> {
        let environment = configuration
            .environments
            .into_iter()
            .find(|e| e.environment_id == environment_id)
            .ok_or(ConfigurationAccessError::EnvironmentNotFound {
                environment_id: environment_id.to_string(),
            })?;
        // FIXME: why not filtering for collection here?

        let mut features = HashMap::new();
        for feature in environment.features {
            features.insert(feature.feature_id.clone(), feature);
        }

        let mut properties = HashMap::new();
        for property in environment.properties {
            properties.insert(property.property_id.clone(), property);
        }

        let mut segments = HashMap::new();
        for segment in configuration.segments {
            segments.insert(segment.segment_id.clone(), segment.clone());
        }
        Ok(ConfigurationSnapshot {
            features: features,
            properties: properties,
            segments: segments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tests::example_configuration_enterprise;
    use crate::models::Configuration;
    use rstest::*;

    #[rstest]
    fn test_filter_configurations(example_configuration_enterprise: Configuration) {
        let result =
            ConfigurationSnapshot::new("does_for_sure_not_exist", example_configuration_enterprise);
        assert!(result.is_err());

        assert!(matches!(
                result.unwrap_err(),
                ConfigurationAccessError::EnvironmentNotFound{ref environment_id}
                if environment_id == "does_for_sure_not_exist"));
    }
}
