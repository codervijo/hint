use reqwest::Error;
use serde::Deserialize;

const BASE_URL: &str = "https://hacker-news.firebaseio.com/v0/";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Story {
    pub id: u64,
    pub by: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub score: Option<u32>,
    pub time: Option<u64>,
    pub descendants: Option<u32>,
}

pub async fn fetch_top_stories() -> Result<Vec<u64>, Error> {
    let url = format!("{BASE_URL}topstories.json");
    let response = reqwest::get(&url).await?;
    let story_ids: Vec<u64> = response.json().await?;
    Ok(story_ids)
}

pub async fn fetch_story_details(story_id: u64) -> Result<Story, Error> {
    let url = format!("{BASE_URL}item/{story_id}.json");
    let response = reqwest::get(&url).await?;
    let story: Story = response.json().await?;
    Ok(story)
}

#[allow(dead_code)]
pub async fn fetch_new_stories() -> Result<Vec<u64>, Error> {
    let url = format!("{BASE_URL}newstories.json");
    let response = reqwest::get(&url).await?;
    let story_ids: Vec<u64> = response.json().await?;
    Ok(story_ids)
}

#[allow(dead_code)]
pub async fn fetch_ask_stories() -> Result<Vec<u64>, Error> {
    let url = format!("{BASE_URL}askstories.json");
    let response = reqwest::get(&url).await?;
    let story_ids: Vec<u64> = response.json().await?;
    Ok(story_ids)
}

#[allow(dead_code)]
pub async fn fetch_show_stories() -> Result<Vec<u64>, Error> {
    let url = format!("{BASE_URL}showstories.json");
    let response = reqwest::get(&url).await?;
    let story_ids: Vec<u64> = response.json().await?;
    Ok(story_ids)
}

#[allow(dead_code)]
pub async fn fetch_job_stories() -> Result<Vec<u64>, Error> {
    let url = format!("{BASE_URL}jobstories.json");
    let response = reqwest::get(&url).await?;
    let story_ids: Vec<u64> = response.json().await?;
    Ok(story_ids)
}
