use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct SearchModsInput {
    /// Optional text to match against mod names. Omit to browse mods.
    pub query: Option<String>,
    /// Target platform. Defaults to windows.
    #[serde(default)]
    pub platform: ModPlatform,
    /// Result ordering. Defaults to updated.
    #[serde(default)]
    pub sort: ModSort,
    /// Number of results. Defaults to 10 and cannot exceed 20.
    pub limit: Option<u32>,
    /// Continuation offset returned by a previous call.
    pub cursor: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct GetModInput {
    /// Numeric mod.io identifier of the mod.
    pub mod_id: u64,
    /// Target platform. Defaults to windows.
    #[serde(default)]
    pub platform: ModPlatform,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModPlatform {
    #[default]
    Windows,
    Mac,
    Ps5,
    Xboxseriesx,
}

impl ModPlatform {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Mac => "mac",
            Self::Ps5 => "ps5",
            Self::Xboxseriesx => "xboxseriesx",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ModSort {
    #[default]
    Updated,
    Newest,
    Downloads,
    Popular,
    Rating,
    Name,
}

impl ModSort {
    pub(crate) fn api_value(self) -> &'static str {
        match self {
            Self::Updated => "-date_updated",
            Self::Newest => "-date_live",
            Self::Downloads => "-downloads_total",
            Self::Popular => "-subscribers_total",
            Self::Rating => "-ratings_weighted_aggregate",
            Self::Name => "name",
        }
    }
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModIoAttribution {
    pub name: String,
    pub url: String,
    pub terms_url: String,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModAuthor {
    pub id: Option<u64>,
    pub username: Option<String>,
    pub profile_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModStats {
    pub downloads_total: Option<u64>,
    pub subscribers_total: Option<u64>,
    pub ratings_total: Option<u64>,
    pub ratings_percentage_positive: Option<u32>,
    pub ratings_weighted_aggregate: Option<f64>,
    pub ratings_display_text: Option<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModSummary {
    pub id: u64,
    pub name: String,
    pub summary: String,
    pub profile_url: String,
    pub author: Option<ModAuthor>,
    pub date_updated: Option<u64>,
    pub date_live: Option<u64>,
    pub logo_url: Option<String>,
    pub tags: Vec<String>,
    pub platforms: Vec<String>,
    pub current_version: Option<String>,
    pub stats: ModStats,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct SearchModsResponse {
    pub query: Option<String>,
    pub platform: ModPlatform,
    pub sort: ModSort,
    pub results: Vec<ModSummary>,
    pub result_total: u64,
    pub next_cursor: Option<u64>,
    pub attribution: ModIoAttribution,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModFileSummary {
    pub id: u64,
    pub version: Option<String>,
    pub filename: Option<String>,
    pub changelog: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_uncompressed: Option<u64>,
    pub date_updated: Option<u64>,
    pub virus_status: Option<u32>,
    pub virus_positive: Option<u32>,
    pub platforms: Vec<String>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ModDetailResponse {
    pub id: u64,
    pub name: String,
    pub name_id: String,
    pub summary: String,
    pub description: String,
    pub profile_url: String,
    pub homepage_url: Option<String>,
    pub author: Option<ModAuthor>,
    pub date_added: Option<u64>,
    pub date_updated: Option<u64>,
    pub date_live: Option<u64>,
    pub logo_url: Option<String>,
    pub image_urls: Vec<String>,
    pub youtube_urls: Vec<String>,
    pub tags: Vec<String>,
    pub platforms: Vec<String>,
    pub maturity_options: Vec<String>,
    pub credit_options: Vec<String>,
    pub current_file: Option<ModFileSummary>,
    pub has_dependencies: bool,
    pub stats: ModStats,
    pub attribution: ModIoAttribution,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ListApiResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub result_count: u64,
    #[serde(default)]
    pub result_offset: u64,
    #[serde(default)]
    pub result_total: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModApi {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_id: String,
    #[serde(default)]
    pub summary: String,
    pub description_plaintext: Option<String>,
    #[serde(default)]
    pub profile_url: String,
    pub homepage_url: Option<String>,
    pub submitted_by: Option<UserApi>,
    pub date_added: Option<u64>,
    pub date_updated: Option<u64>,
    pub date_live: Option<u64>,
    #[serde(default)]
    pub maturity_option: u32,
    #[serde(default)]
    pub credit_options: u32,
    pub logo: Option<LogoApi>,
    #[serde(default)]
    pub media: MediaApi,
    pub modfile: Option<ModfileApi>,
    #[serde(default)]
    pub dependencies: bool,
    #[serde(default)]
    pub platforms: Vec<ModPlatformApi>,
    #[serde(default)]
    pub tags: Vec<ModTagApi>,
    #[serde(default)]
    pub stats: ModStatsApi,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct UserApi {
    pub id: Option<u64>,
    pub username: Option<String>,
    pub profile_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct LogoApi {
    pub original: Option<String>,
    pub thumb_640x360: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct MediaApi {
    #[serde(default)]
    pub youtube: Vec<String>,
    #[serde(default)]
    pub images: Vec<ImageApi>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ImageApi {
    pub original: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModPlatformApi {
    #[serde(default)]
    pub platform: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModTagApi {
    #[serde(default)]
    pub name: String,
    pub name_localized: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct ModStatsApi {
    pub downloads_total: Option<u64>,
    pub subscribers_total: Option<u64>,
    pub ratings_total: Option<u64>,
    pub ratings_percentage_positive: Option<u32>,
    pub ratings_weighted_aggregate: Option<f64>,
    pub ratings_display_text: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModfileApi {
    pub id: Option<u64>,
    pub version: Option<String>,
    pub filename: Option<String>,
    pub changelog: Option<String>,
    pub filesize: Option<u64>,
    pub filesize_uncompressed: Option<u64>,
    pub date_updated: Option<u64>,
    pub virus_status: Option<u32>,
    pub virus_positive: Option<u32>,
    #[serde(default)]
    pub platforms: Vec<ModfilePlatformApi>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ModfilePlatformApi {
    #[serde(default)]
    pub platform: String,
}
