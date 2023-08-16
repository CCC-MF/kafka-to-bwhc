/*
 * This file is part of ETL-Processor
 *
 * Copyright (c) 2023  Comprehensive Cancer Center Mainfranken
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::str::FromStr;

use serde::Deserialize;
use serde_json::Value;
use crate::resources::mtbfile::MTBFileWithConsent;

#[derive(Deserialize)]
pub struct Request {

    #[serde(alias = "requestId")]
    request_id: String,

    content: Value

}

impl FromStr for Request {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str(s) {
            Ok(o) => Ok(o),
            Err(_) => Err(())
        }
    }
}

impl Request {

    pub fn can_parse(s: &str) -> bool {
        match Self::from_str(s) {
            Ok(request) => {
                MTBFileWithConsent::from_str(request.content_string().as_str()).is_ok()
            },
            Err(_) => false
        }
    }

    pub fn request_id(&self) -> String {
        self.request_id.to_string()
    }

    pub fn content_string(&self) -> String {
        self.content.to_string()
    }

    pub fn has_consent(&self) -> bool {
        match MTBFileWithConsent::from_str(self.content.to_string().as_str()) {
            Ok(mtbfile) => mtbfile.has_consent(),
            _ => false
        }
    }

    pub fn patient_id(&self) -> String {
        match MTBFileWithConsent::from_str(self.content.to_string().as_str()) {
            Ok(mtbfile) => mtbfile.patient_id(),
            _ => String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::resources::request::Request;

    #[test]
    fn should_return_that_request_can_be_parsed() {
        let jsonstr = r#"
           {
                "request_id": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "active"
                    }
                }
           }
        "#;

        assert!(Request::can_parse(jsonstr))
    }

    #[test]
    fn should_return_that_request_can_be_parsed_with_request_id_alias() {
        let jsonstr = r#"
           {
                "requestId": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "active"
                    }
                }
           }
        "#;

        assert!(Request::can_parse(jsonstr))
    }

    #[test]
    fn should_return_that_request_without_request_id_cannot_be_parsed() {
        let jsonstr = r#"
           {
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "active"
                    }
                }
           }
        "#;

        assert_eq!(Request::can_parse(jsonstr), false)
    }

    #[test]
    fn should_return_that_request_without_content_cannot_be_parsed() {
        let jsonstr = r#"
           {
                "requestId": "test123456789"
           }
        "#;

        assert_eq!(Request::can_parse(jsonstr), false)
    }


    #[test]
    fn should_parse_request_and_return_mtb_file_consent() {
        let jsonstr = r#"
           {
                "requestId": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "active"
                    }
                }
           }
        "#;

        let actual = Request::from_str(jsonstr);

        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap().has_consent(),
            true
        )
    }

    #[test]
    fn should_parse_request_and_return_mtb_file_rejected_consent() {
        let jsonstr = r#"
           {
                "requestId": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "rejected"
                    }
                }
           }
        "#;

        let actual = Request::from_str(jsonstr);

        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap().has_consent(),
            false
        )
    }

    #[test]
    fn should_parse_request_and_return_content_as_string() {
        let jsonstr = r#"
           {
                "requestId": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "rejected"
                    }
                }
           }
        "#;

        let actual = Request::from_str(jsonstr);

        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap().content_string(),
            r#"{"consent":{"id":"TESTID1234","patient":"TESTPATIENT1234","status":"rejected"}}"#.to_string()
        )
    }

    #[test]
    fn should_parse_request_and_return_request_id_as_string() {
        let jsonstr = r#"
           {
                "requestId": "request0123456789",
                "content": {
                    "consent": {
                        "id": "TESTID1234",
                        "patient": "TESTPATIENT1234",
                        "status": "rejected"
                    }
                }
           }
        "#;

        let actual = Request::from_str(jsonstr);

        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap().request_id(),
            "request0123456789".to_string()
        )
    }

}