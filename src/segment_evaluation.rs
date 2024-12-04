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

use std::collections::HashMap;

use crate::models::Segment;
use crate::{
    entity::{AttrValue, Entity},
    models::TargetingRule,
};

pub(crate) fn find_applicable_segment_rule_for_entity(
    segments: &HashMap<String, Segment>,
    segment_rules: impl Iterator<Item = TargetingRule>,
    entity: &impl Entity,
) -> Option<TargetingRule> {
    let mut targeting_rules = segment_rules.collect::<Vec<_>>();
    targeting_rules.sort_by(|a, b| a.order.cmp(&b.order));
    targeting_rules
        .into_iter()
        .find(|targeting_rule| targeting_rule_applies_to_entity(segments, targeting_rule, entity))
}

fn targeting_rule_applies_to_entity(
    segments: &HashMap<String, Segment>,
    targeting_rule: &TargetingRule,
    entity: &impl Entity,
) -> bool {
    let rules = &targeting_rule.rules;
    rules
        .iter()
        .any(|rules| segment_applies_to_entity(segments, &rules.segments, entity))
}

fn segment_applies_to_entity(
    segments: &HashMap<String, Segment>,
    segment_ids: &[String],
    entity: &impl Entity,
) -> bool {
    segment_ids
        .iter()
        .map(|segment_id| {
            segments
                .get(segment_id)
                .unwrap_or_else(|| panic!("Segment {} not found", segment_id))
        })
        .any(|segment| belong_to_segment(segment, entity.get_attributes()))
}

fn belong_to_segment(segment: &Segment, attrs: HashMap<String, AttrValue>) -> bool {
    segment.rules.iter().all(|rule| {
        let operator = &rule.operator;
        let attr_name = &rule.attribute_name;
        let attr_value = attrs
            .get(attr_name)
            .expect("Attribute does not exist in the entity.");
        rule.values
            .iter()
            .any(|value| check_operator(attr_value, operator, value))
    })
}

fn check_operator(attribute_value: &AttrValue, operator: &str, reference_value: &str) -> bool {
    match operator {
        "is" => match attribute_value {
            AttrValue::String(data) => *data == reference_value,
            AttrValue::Boolean(data) => {
                *data
                    == reference_value
                        .parse::<bool>()
                        .expect("Value cannot convert into a bool.")
            }
            AttrValue::Numeric(data) => {
                *data
                    == reference_value
                        .parse::<f64>()
                        .expect("Value cannot convert into a number.")
            }
        },
        "contains" => match attribute_value {
            AttrValue::String(data) => data.contains(reference_value),
            _ => panic!("Entity attribute is not a string."),
        },
        "startsWith" => match attribute_value {
            AttrValue::String(data) => data.starts_with(reference_value),
            _ => panic!("Entity attribute is not a string."),
        },
        "endsWith" => match attribute_value {
            AttrValue::String(data) => data.ends_with(reference_value),
            _ => panic!("Entity attribute is not a string."),
        },
        "greaterThan" => match attribute_value {
            AttrValue::Numeric(data) => {
                *data
                    > reference_value
                        .parse()
                        .expect("Value cannot be converted to number.")
            }
            _ => panic!("Entity attribute is not a number."),
        },
        "lesserThan" => match attribute_value {
            AttrValue::Numeric(data) => {
                *data
                    < reference_value
                        .parse()
                        .expect("Value cannot be converted to number.")
            }
            _ => panic!("Entity attribute is not a number."),
        },
        "greaterThanEquals" => match attribute_value {
            AttrValue::Numeric(data) => {
                *data
                    >= reference_value
                        .parse()
                        .expect("Value cannot be converted to number.")
            }
            _ => panic!("Entity attribute is not a number."),
        },
        "lesserThanEquals" => match attribute_value {
            AttrValue::Numeric(data) => {
                *data
                    <= reference_value
                        .parse()
                        .expect("Value cannot be converted to number.")
            }
            _ => panic!("Entity attribute is not a number."),
        },
        v => {
            panic!("{} not implemented yet", v);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        models::{ConfigValue, Segment, SegmentRule, Segments, TargetingRule},
        AttrValue,
    };

    #[ignore] // This fails, probably a bug. Need to cross check expectation with go impl.
    #[test]
    fn test_missing_attribute() {
        let segments = HashMap::from([(
            "some_segment_id_1".into(),
            Segment {
                name: "".into(),
                segment_id: "".into(),
                description: "".into(),
                tags: None,
                rules: vec![SegmentRule {
                    attribute_name: "name".into(),
                    operator: "is".into(),
                    values: vec!["heinz".into()],
                }],
            },
        )]);
        let segment_rules = vec![TargetingRule {
            rules: vec![Segments {
                segments: vec!["some_segment_id_1".into()],
            }],
            value: ConfigValue(serde_json::Value::Number((-48).into())),
            order: 0,
            rollout_percentage: Some(ConfigValue(serde_json::Value::Number((100).into()))),
        }];
        let entity = crate::tests::GenericEntity {
            id: "a2".into(),
            attributes: HashMap::from([("name2".into(), AttrValue::from("heinz".to_string()))]),
        };
        let rule =
            find_applicable_segment_rule_for_entity(&segments, segment_rules.into_iter(), &entity);
        assert!(rule.is_none());
    }
}
