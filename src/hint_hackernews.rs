use std::fmt;
use crate::hnreader;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HnStoryType {
    Story,
    Ask,
    Comment,
    Job,
    Poll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HnStory {
    id: u32,
    author: String,
    title: String,
    url: Option<String>,
    hntype: HnStoryType,
}

impl HnStoryType {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            HnStoryType::Story => "story".to_string(),
            HnStoryType::Ask => "ask".to_string(),
            HnStoryType::Comment => "comment".to_string(),
            HnStoryType::Job => "job".to_string(),
            HnStoryType::Poll => "poll".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn from_string(typev: String) -> Self {
        match typev.as_str() {
            "story" => HnStoryType::Story,
            "ask" => HnStoryType::Ask,
            "comment" => HnStoryType::Comment,
            "job" => HnStoryType::Job,
            "poll" => HnStoryType::Poll,
            &_ => todo!(),
        }
    }
}

impl HnStory {
    #[allow(dead_code)]
    pub fn new(id: String, author: String, title: String, url: Option<String>, typev: String) -> Self {
        Self {
            id: id.parse().unwrap_or(0),
            author,
            title,
            url,
            hntype: HnStoryType::from_string(typev),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HnStoryList {
    storyidlist: Vec<u64>,
    storylist: Vec<HnStory>,
    expected_len: u32,
    read_len: usize,
}

impl HnStoryList {
    pub async fn new() -> Self {
        match hnreader::fetch_top_stories().await {
            Ok(story_ids) => {
                Self {
                    storyidlist: story_ids.clone(),
                    storylist: vec!(),
                    expected_len: 10,
                    read_len: story_ids.len(),
                }
            },
            Err(err) => {
                eprintln!("Failed to fetch top stories: {}", err);
                // Return a default value for `HnStoryList` in case of an error
                Self {
                    storyidlist: vec!(),  // Default empty list
                    storylist: vec!(),
                    expected_len: 10,
                    read_len: 0,
                }
            },
        }
    }
}

impl fmt::Debug for HnStoryList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HnStoryList")
            .field("storyidlist", &self.storyidlist)
            .field("storylist", &self.storylist)
            .field("expected_len", &self.expected_len)
            .field("read_len", &self.read_len)
            .finish()
    }
}