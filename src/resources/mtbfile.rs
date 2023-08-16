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

#[derive(Deserialize)]
pub struct MTBFileWithConsent {
    consent: Consent
}

impl MTBFileWithConsent {
    pub fn has_consent(&self) -> bool {
        self.consent.status == Status::Active
    }

    pub fn patient_id(&self) -> String {
        self.consent.patient.clone()
    }
}

impl FromStr for MTBFileWithConsent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str(s) {
            Ok(o) => Ok(o),
            Err(_) => Err(())
        }
    }
}

#[derive(Deserialize)]
struct Consent {
    status: Status,
    patient: String
}

#[derive(Deserialize, PartialEq)]
enum Status {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "rejected")]
    Rejected
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::resources::mtbfile::MTBFileWithConsent;

    #[test]
    fn should_parse_mtb_file_with_active_consent() {
        let jsonstr = r#"
           {
                "consent": {
                    "id": "TESTID1234",
                    "patient": "TESTPATIENT1234",
                    "status": "active"
                }
           }
        "#;

        let actual = MTBFileWithConsent::from_str(jsonstr);

        assert!(actual.is_ok())
    }

    #[test]
    fn should_parse_mtb_file_with_rejected_consent() {
        let jsonstr = r#"
           {
                "consent": {
                    "id": "TESTID1234",
                    "patient": "TESTPATIENT1234",
                    "status": "rejected"
                }
           }
        "#;

        let actual = MTBFileWithConsent::from_str(jsonstr);

        assert!(actual.is_ok())
    }

}