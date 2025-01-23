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
pub struct HnStory {
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

    #[allow(dead_code)]
    pub fn author(&self) -> &str {
        &self.author
    }

    #[allow(dead_code)]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[allow(dead_code)]
    pub fn details(&self) -> &str {
        "Details of title"
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HnStoryList {
    storyidlist: Vec<u64>,
    storylist: Vec<HnStory>,
    story_writer: u32,
    story_maxlen: usize,
}

// Define the Iterator for HnStoryList
pub struct HnStoryListIter<'a> {
    index: usize,
    storylist: &'a [HnStory],
}

impl<'a> Iterator for HnStoryListIter<'a> {
    type Item = &'a HnStory;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.storylist.len() {
            let story = &self.storylist[self.index];
            self.index += 1;
            Some(story)
        } else {
            None
        }
    }
}

impl HnStoryList {
    pub async fn new() -> Self {
        match hnreader::fetch_top_stories().await {
            Ok(story_ids) => {
                let mut idx = 0;
                let mut storydets = vec!();
                for (i, sid) in story_ids.iter().enumerate() {
                    if i > 10 {
                        break;
                    }
                    let mut title = String::from("abc");
                    let mut url = String::from("hcker");
                    match hnreader::fetch_story_details(*sid).await {
                        Ok(story) => {
                            //println!("Story Details: {:?}", story);
                            title = story.title.clone().unwrap_or_else(|| String::from("Untitled"));
                            url = story.url.clone().unwrap_or_else(|| String::from("http://example.com"));
                        }
                        Err(err) => eprintln!("Failed to fetch story details: {}", err),
                    }
                    //println!("\n");
                    storydets.push(HnStory {
                        id: i as u32,
                        author: String::from("Unknown"),
                        title,
                        url: Some(url),
                        hntype: HnStoryType::Story,
                    });
                    idx += 1;
                }
                Self {
                    storyidlist: story_ids.clone(),
                    storylist: storydets,
                    story_writer: idx,
                    story_maxlen: story_ids.len(),
                }
            },
            Err(err) => {
                eprintln!("Failed to fetch top stories: {}", err);
                // Return a default value for `HnStoryList` in case of an error
                Self {
                    storyidlist: vec!(),  // Default empty list
                    storylist: vec!(),
                    story_writer: 0,
                    story_maxlen: 0,
                }
            },
        }
    }

    pub fn iter(&self) -> HnStoryListIter {
        HnStoryListIter {
            index: 0,
            storylist: &self.storylist,
        }
    }

    pub fn is_filled(&self) -> bool {
        self.story_writer == self.story_maxlen as u32
    }
}

impl fmt::Debug for HnStoryList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HnStoryList")
            .field("storyidlist", &self.storyidlist)
            .field("storylist", &self.storylist)
            .field("expected_len", &self.story_writer)
            .field("read_len", &self.story_maxlen)
            .finish()
    }
}