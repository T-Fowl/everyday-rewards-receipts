use std::path::Path;

use thiserror::Error;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, USER_AGENT};
use serde_json::json;

use crate::EverydayRewardsError::UnknownError;
use crate::models::{ApiError, ApiResponse, ReceiptDetailsResponse, RewardsActivityFeedResponse, RtlRewardsActivity};

const CLIENT_ID: &str = "client_id";
const WEB_CLIENT_ID: &str = "8h41mMOiDULmlLT28xKSv5ITpp3XBRvH";
const API_VERSION: &str = "api-version";

#[derive(Debug)]
pub struct ValueWithSource<T> {
    pub value: T,
    pub source: String,
}

impl<T> AsRef<T> for ValueWithSource<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

pub struct EverydayRewardsClient {
    client: Client,
}

#[derive(Error, Debug)]
pub enum EverydayRewardsError {
    #[error("everydayrewards api responded with the following errors: {0:?}")]
    ApiError(Vec<ApiError>),

    #[error("network error")]
    NetworkError(#[from] reqwest::Error),

    #[error("parsing error")]
    ParseError(#[from] serde_json::Error),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("unknown error: {0}")]
    UnknownError(String),
}

impl EverydayRewardsClient {
    pub fn create(token: &str) -> Result<EverydayRewardsClient, EverydayRewardsError> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
        headers.insert(CLIENT_ID, WEB_CLIENT_ID.parse().unwrap());
        headers.insert(API_VERSION, "2".parse().unwrap());
        headers.insert(USER_AGENT, "Everyday Rewards Receipts Downloader by Thomas Fowler (https://github.com/T-Fowl/everyday-rewards-receipts)".parse().unwrap());

        let http = reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        let client = EverydayRewardsClient {
            client: http,
        };

        Ok(client)
    }

    pub fn rewards_activity_feed(&self, page_token: Option<&str>) -> Result<ValueWithSource<RewardsActivityFeedResponse>, EverydayRewardsError> {
        let page_token = page_token.unwrap_or("FIRST_PAGE");

        let query = include_str!("RewardsActivityFeed.graphql");
        let query = query.replace("FIRST_PAGE", page_token);

        let body = json!({
            "query": query,
        });

        let text = self.client.post("https://apigee-prod.api-wr.com/wx/v1/bff/graphql")
            .body(body.to_string())
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .send()?
            .text()?;

        let response = serde_json::from_str::<ApiResponse>(text.as_str())?;

        if let Some(errors) = response.errors {
            return Err(EverydayRewardsError::ApiError(errors));
        }

        if let Some(data) = response.data {
            let response = serde_json::from_str::<RewardsActivityFeedResponse>(data.get())?;
            return Ok(ValueWithSource {
                value: response,
                source: data.to_string(),
            });
        }

        return Err(UnknownError(format!("Unknown response: {text}")));
    }

    pub fn transaction_details(&self, receipt_key: &str) -> Result<ValueWithSource<ReceiptDetailsResponse>, EverydayRewardsError> {
        let body = json!({
            "receiptKey": receipt_key,
        });

        let text = self.client.post("https://api.woolworthsrewards.com.au/wx/v1/rewards/member/ereceipts/transactions/details")
            .body(body.to_string())
            .header(CONTENT_TYPE, "application/json;charset=UTF-8").send()?.text()?;

        let response: ApiResponse = serde_json::from_str(text.as_str())?;

        if let Some(errors) = response.errors {
            return Err(EverydayRewardsError::ApiError(errors));
        }

        if let Some(data) = response.data {
            let response = serde_json::from_str::<ReceiptDetailsResponse>(data.get())?;
            return Ok(ValueWithSource {
                value: response,
                source: data.to_string(),
            });
        }

        return Err(UnknownError(format!("Unknown response: {text}")));
    }

    pub fn download_receipt<P: AsRef<Path>>(&self, download_url: &str, path: P) -> Result<(), EverydayRewardsError> {
        let body = json!({
            "downloadUrl": download_url,
        });

        let request = self.client.post("https://api.woolworthsrewards.com.au/wx/v1/rewards/member/ereceipts/transactions/details/download")
            .body(body.to_string())
            .header(CONTENT_TYPE, "application/json;charset=UTF-8");

        let response = request.send()?;

        let bytes = response.bytes()?;

        std::fs::write(path, bytes)?;

        Ok(())
    }
}


pub struct ActivityFeedIterator<'client> {
    client: &'client EverydayRewardsClient,
    page: Option<String>,
}

impl<'client> ActivityFeedIterator<'client> {
    pub fn create(client: &EverydayRewardsClient) -> ActivityFeedIterator {
        ActivityFeedIterator {
            client,
            page: Some(String::from("FIRST_PAGE")),
        }
    }
}

impl<'client> Iterator for ActivityFeedIterator<'client> {
    type Item = Result<RtlRewardsActivity, EverydayRewardsError>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.page {
            None => None,
            Some(page) => {
                let response = self.client.rewards_activity_feed(Some(page));

                match response {
                    Err(e) => {
                        self.page = None; // Stop iteration
                        Some(Err(e))
                    }
                    Ok(res) => {
                        let res = res.value;

                        // TODO: Need to brush up on rust ownership
                        self.page = res.rtl_rewards_activity_feed.list.next_page_token.clone();
                        Some(Ok(res.rtl_rewards_activity_feed.list))
                    }
                }
            }
        }
    }
}