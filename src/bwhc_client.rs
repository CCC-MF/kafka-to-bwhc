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

use std::env;
use std::time::Duration;
use crate::AppError;
use crate::AppError::{HttpError, MissingConfig};

pub struct HttpResponse {
    pub status_code: u16,
    pub status_body: String,
}

pub struct BwhcClient;

impl BwhcClient {
    pub async fn send_mtb_file(content: &str) -> Result<HttpResponse, AppError> {
        let uri = env::var("APP_REST_URI").map_err(|e| MissingConfig(e.to_string()))?;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/MTBFile", uri))
            .body(content.to_string())
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| HttpError(e.to_string()))?;

        Ok(
            HttpResponse { status_code: response.status().as_u16(), status_body: response.text().await.unwrap_or(String::new()) }
        )
    }

    pub async fn send_delete(patient_id: &str) -> Result<HttpResponse, AppError> {
        let uri = env::var("APP_REST_URI").map_err(|e| MissingConfig(e.to_string()))?;

        let client = reqwest::Client::new();
        let response = client
            .delete(format!("{}/MTBFile/{}", uri, patient_id))
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| HttpError(e.to_string()))?;

        Ok(
            HttpResponse { status_code: response.status().as_u16(), status_body: response.text().await.unwrap_or(String::new()) }
        )
    }
}