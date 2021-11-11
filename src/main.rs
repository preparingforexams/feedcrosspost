use std::env;
use std::time::Duration;
use chrono::{DateTime, NaiveDateTime, Utc};
use roux::{Me, Reddit};
use roux::util::RouxError;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let time_offset_minutes: i64 = env::var("TIME_OFFSET_MINUTES").unwrap_or("30".parse()?).parse::<i64>()?;
    let client = get_reddit_client().await?;
    let subreddit = env::var("SUBREDDIT_NAME")?;

    for item in get_feed_items(time_offset_minutes).await? {
        item.submit_to_reddit(&client, subreddit.as_str()).await;
    }

    return Ok(());
}

#[derive(Debug, Deserialize)]
struct Feed {
    items: Vec<RawFeedItem>,
}

#[derive(Debug, Deserialize)]
struct RawFeedItem {
    title: String,
    url: String,
    date_published: String,
}

impl RawFeedItem {
    pub fn convert_time(&self) -> anyhow::Result<NaiveDateTime> {
        Ok(DateTime::parse_from_rfc3339(self.date_published.as_str())?.naive_utc())
    }
}

impl TryInto<FeedItem> for RawFeedItem {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<FeedItem, Self::Error> {
        let date_published = self.convert_time()?;
        Ok(FeedItem {
            title: self.title,
            url: self.url,
            date_published,
        })
    }
}

#[derive(Debug)]
struct FeedItem {
    title: String,
    url: String,
    date_published: NaiveDateTime,
}

impl FeedItem {
    pub fn is_new(&self, offset_minutes: i64) -> bool {
        let start = Utc::now() - chrono::Duration::minutes(offset_minutes);

        return self.date_published > start.naive_utc();
    }

    pub async fn submit_to_reddit(&self, client: &Me, subreddit: &str) {
        println!("submitting: {:#?}", self);
        let submission = client.submit_link(self.title.as_str(), self.url.as_str(), subreddit).await;
        if submission.is_ok() {
            println!("Failed to submit item ({:#?}):\n {:#?}", self, submission)
        }
    }
}

async fn get_feed_items(offset_minutes: i64) -> Result<Vec<FeedItem>, anyhow::Error> {
    let sleep_for = Duration::from_millis(10);

    // long polling: 10 millisecond
    // super useful stuff right here
    for _ in 0..1u64 {
        let feed = reqwest::get("https://kerkour.com/feed.json")
            .await?
            .json::<Feed>()
            .await?;

        let posts: Vec<FeedItem> = feed
            .items
            .into_iter()
            .filter_map(|item| item.try_into().ok())
            .filter(|item: &FeedItem| item.is_new(offset_minutes))
            .collect();
        if posts.len() != 0 {
            return Ok(posts);
        }

        tokio::time::sleep(sleep_for).await;
    }

    // return an empty response
    Ok(Vec::new().into())
}

async fn get_reddit_client() -> anyhow::Result<Me> {
    let client: Result<Me, RouxError> = Reddit::new(env::var("USER_AGENT")?.as_str(), env::var("CLIENT_ID")?.as_str(), env::var("CLIENT_SECRET")?.as_str())
        .username(env::var("REDDIT_USERNAME")?.as_str())
        .password(env::var("REDDIT_PASSWORD")?.as_str())
        .login()
        .await;

    Ok(client?)
}
