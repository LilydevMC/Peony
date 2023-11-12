use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub prerelease: bool
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseResponse {
    pub url: String,
    pub html_url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub tarball_url: Option<String>,
    pub zipball_url: Option<String>,
    pub id: i32,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub author: GithubAuthor,
    pub assets: Vec<GithubAsset>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub mentions_count: Option<i32>,
    pub discussion_url: Option<String>,
    pub reactions: Option<GithubReactions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
    pub login: String,
    pub id: i32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_url: Option<String>,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_string: String,
    pub site_admin: bool,
    pub starred_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubAsset {
    pub url: String,
    pub browser_download_url: String,
    pub id: i32,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub state: String,
    pub content_type: String,
    pub size: i32,
    pub download_count: i32,
    pub created_at: String,
    pub updated_at: String,
    pub uploader: Option<GithubAuthor>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubReactions {
    pub url: String,
    pub total_count: i32,
    #[serde(rename = "+1")]
    pub plus_one: i32,
    #[serde(rename = "+1")]
    pub minus_one: i32,
    pub laugh: i32,
    pub confused: i32,
    pub heart: i32,
    pub hooray: i32,
    pub eyes: i32,
    pub rocket: i32,
}
