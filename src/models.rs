use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<'a> {
    #[serde(borrow)]
    pub data: Option<&'a RawValue>,
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub status: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardsActivityFeedResponse {
    #[serde(rename = "rtlRewardsActivityFeed")]
    pub rtl_rewards_activity_feed: RtlRewardsActivityFeed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RtlRewardsActivityFeed {
    pub list: RtlRewardsActivity,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RtlRewardsActivity {
    // NOTE: There are other types but I have changed the GraphQL query to only return this type
    // If woolies starts using prepared queries in the future - this will have to be changed into an enum
    pub groups: Vec<RewardsActivityFeedGroup>,

    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RewardsActivityFeedGroup {
    pub id: String,
    pub title: String,
    pub items: Option<Vec<Item>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub receipt: Option<ItemReceipt>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemReceipt {
    #[serde(rename = "receiptId")]
    pub receipt_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiptDetailsResponse {
    #[serde(rename = "receiptDetails")]
    pub receipt_details: ReceiptDetails,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiptDetails {
    pub download: ReceiptDownload,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceiptDownload {
    pub filename: String,
    pub url: String,
}