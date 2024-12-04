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

#[derive(PartialEq, Debug)]
pub struct NumericValue(pub(crate) serde_json::Value);

impl NumericValue {
    pub fn as_i64(&self) -> Option<i64> {
        self.0.as_i64()
    }
    pub fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }
    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }
}

#[derive(PartialEq, Debug)]
pub enum Value {
    Numeric(NumericValue),
    String(String),
    Boolean(bool),
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_numeric() {
        let value = Value::Numeric(NumericValue(serde_json::Value::Number(42.into())));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_f64().unwrap() == 42f64));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_i64().unwrap() == 42i64));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_u64().unwrap() == 42u64));

        let json_value = serde_json::Value::from(-42.0f64);
        let value = Value::Numeric(NumericValue(json_value));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_f64().unwrap() == -42f64));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_i64().is_none()));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_u64().is_none()));

        let json_value = serde_json::Value::from(-42i64);
        let value = Value::Numeric(NumericValue(json_value));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_f64().unwrap() == -42f64));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_i64().unwrap() == -42i64));
        assert!(matches!(value, Value::Numeric(ref v) if v.as_u64().is_none()));
    }
}
