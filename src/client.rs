use std::fmt::{Display, Formatter};
use std::path::Path;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, USER_AGENT};

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

#[derive(Debug)]
pub enum EverydayRewardsError {
    ApiError(Vec<ApiError>),
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    IoError(std::io::Error),
    UnknownError(String),
}

impl Display for EverydayRewardsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EverydayRewardsError {}

impl From<std::io::Error> for EverydayRewardsError {
    fn from(error: std::io::Error) -> Self {
        EverydayRewardsError::IoError(error)
    }
}

impl From<serde_json::Error> for EverydayRewardsError {
    fn from(error: serde_json::Error) -> Self {
        EverydayRewardsError::ParseError(error)
    }
}

impl From<reqwest::Error> for EverydayRewardsError {
    fn from(error: reqwest::Error) -> Self {
        EverydayRewardsError::NetworkError(error)
    }
}

impl From<Vec<ApiError>> for EverydayRewardsError {
    fn from(errors: Vec<ApiError>) -> Self {
        EverydayRewardsError::ApiError(errors)
    }
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
        let query = format!(r#"query RewardsActivityFeed {{ rtlRewardsActivityFeed(pageToken: \"{page_token}\") {{ list {{ groups {{ ... on RewardsActivityFeedGroup {{ id title items {{ id receipt {{ receiptId }} }} }} }} nextPageToken }} }} }}"#);

        let text = self.client.post("https://apigee-prod.api-wr.com/wx/v1/bff/graphql")
            .body(format!("{{\"query\":\"{query}\"}}"))
            .header(CONTENT_TYPE, "application/json;charset=UTF-8")
            .send()?
            .text()?;

        let response = serde_json::from_str::<ApiResponse>(text.as_str())?;

        if let Some(errors) = response.errors {
            return Err(EverydayRewardsError::from(errors));
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
        let text = self.client.post("https://api.woolworthsrewards.com.au/wx/v1/rewards/member/ereceipts/transactions/details")
            .body(format!("{{\"receiptKey\":\"{receipt_key}\"}}"))
            .header(CONTENT_TYPE, "application/json;charset=UTF-8").send()?.text()?;

        let response: ApiResponse = serde_json::from_str(text.as_str())?;

        if let Some(errors) = response.errors {
            return Err(EverydayRewardsError::from(errors));
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
        let request = self.client.post("https://api.woolworthsrewards.com.au/wx/v1/rewards/member/ereceipts/transactions/details/download")
            .body(format!("{{\"downloadUrl\":\"{download_url}\"}}"))
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