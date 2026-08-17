use crate::{Config, error::ModIoError};

use super::{
    ModIoClient,
    models::{
        GetModInput, ModApi, ModAuthor, ModDetailResponse, ModFileSummary, ModIoAttribution,
        ModStats, ModSummary, SearchModsInput, SearchModsResponse,
    },
};

const MAX_CURSOR: u64 = 100_000;

#[derive(Clone)]
pub struct ModsService {
    client: ModIoClient,
}

impl ModsService {
    pub fn from_config(config: &Config) -> Result<Self, crate::error::AppError> {
        Ok(Self {
            client: ModIoClient::new(config)?,
        })
    }

    pub async fn search(&self, input: SearchModsInput) -> Result<SearchModsResponse, ModIoError> {
        let query = input
            .query
            .as_deref()
            .map(str::trim)
            .filter(|query| !query.is_empty());
        if input.query.is_some() && query.is_none() {
            return Err(ModIoError::InvalidInput(
                "query cannot be empty".to_string(),
            ));
        }
        if query.is_some_and(|query| query.chars().count() > 200) {
            return Err(ModIoError::InvalidInput(
                "query cannot exceed 200 characters".to_string(),
            ));
        }
        let limit = input.limit.unwrap_or(10);
        if !(1..=20).contains(&limit) {
            return Err(ModIoError::InvalidInput(
                "limit must be between 1 and 20".to_string(),
            ));
        }
        let cursor = input.cursor.unwrap_or(0);
        if cursor > MAX_CURSOR {
            return Err(ModIoError::InvalidInput(format!(
                "cursor cannot exceed {MAX_CURSOR}"
            )));
        }

        let response = self
            .client
            .search(query, input.platform, input.sort.api_value(), limit, cursor)
            .await?;
        let next_offset = response.result_offset.saturating_add(response.result_count);
        let next_cursor = (next_offset < response.result_total && next_offset <= MAX_CURSOR)
            .then_some(next_offset);

        Ok(SearchModsResponse {
            query: query.map(str::to_string),
            platform: input.platform,
            sort: input.sort,
            results: response.data.into_iter().map(mod_summary).collect(),
            result_total: response.result_total,
            next_cursor,
            attribution: attribution(),
        })
    }

    pub async fn get(&self, input: GetModInput) -> Result<ModDetailResponse, ModIoError> {
        if input.mod_id == 0 {
            return Err(ModIoError::InvalidInput(
                "mod_id must be greater than zero".to_string(),
            ));
        }
        let item = self.client.get_mod(input.mod_id, input.platform).await?;
        Ok(mod_detail(item))
    }
}

fn mod_summary(item: ModApi) -> ModSummary {
    let tags = tags(&item);
    let platforms = platforms(&item);
    ModSummary {
        id: item.id,
        name: item.name,
        summary: item.summary,
        profile_url: item.profile_url,
        author: item.submitted_by.map(author),
        date_updated: item.date_updated,
        date_live: item.date_live,
        logo_url: item
            .logo
            .and_then(|logo| logo.thumb_640x360.or(logo.original)),
        tags,
        platforms,
        current_version: item.modfile.and_then(|file| file.version),
        stats: stats(item.stats),
    }
}

fn mod_detail(item: ModApi) -> ModDetailResponse {
    let tags = tags(&item);
    let platforms = platforms(&item);
    let logo_url = item
        .logo
        .and_then(|logo| logo.original.or(logo.thumb_640x360));
    let image_urls = item
        .media
        .images
        .into_iter()
        .filter_map(|image| image.original)
        .collect();
    let current_file = item.modfile.and_then(file_summary);

    ModDetailResponse {
        id: item.id,
        name: item.name,
        name_id: item.name_id,
        summary: item.summary,
        description: item.description_plaintext.unwrap_or_default(),
        profile_url: item.profile_url,
        homepage_url: item.homepage_url.filter(|url| !url.is_empty()),
        author: item.submitted_by.map(author),
        date_added: item.date_added,
        date_updated: item.date_updated,
        date_live: item.date_live,
        logo_url,
        image_urls,
        youtube_urls: item.media.youtube,
        tags,
        platforms,
        maturity_options: maturity_options(item.maturity_option),
        credit_options: credit_options(item.credit_options),
        current_file,
        has_dependencies: item.dependencies,
        stats: stats(item.stats),
        attribution: attribution(),
    }
}

fn author(value: super::models::UserApi) -> ModAuthor {
    ModAuthor {
        id: value.id,
        username: value.username,
        profile_url: value.profile_url,
    }
}

fn tags(item: &ModApi) -> Vec<String> {
    item.tags
        .iter()
        .map(|tag| {
            tag.name_localized
                .as_deref()
                .filter(|name| !name.is_empty())
                .unwrap_or(&tag.name)
                .to_string()
        })
        .collect()
}

fn platforms(item: &ModApi) -> Vec<String> {
    item.platforms
        .iter()
        .map(|platform| platform.platform.clone())
        .filter(|platform| !platform.is_empty())
        .collect()
}

fn stats(value: super::models::ModStatsApi) -> ModStats {
    ModStats {
        downloads_total: value.downloads_total,
        subscribers_total: value.subscribers_total,
        ratings_total: value.ratings_total,
        ratings_percentage_positive: value.ratings_percentage_positive,
        ratings_weighted_aggregate: value.ratings_weighted_aggregate,
        ratings_display_text: value.ratings_display_text,
    }
}

fn file_summary(value: super::models::ModfileApi) -> Option<ModFileSummary> {
    Some(ModFileSummary {
        id: value.id?,
        version: value.version,
        filename: value.filename,
        changelog: value.changelog,
        filesize: value.filesize,
        filesize_uncompressed: value.filesize_uncompressed,
        date_updated: value.date_updated,
        virus_status: value.virus_status,
        virus_positive: value.virus_positive,
        platforms: value
            .platforms
            .into_iter()
            .map(|platform| platform.platform)
            .filter(|platform| !platform.is_empty())
            .collect(),
    })
}

fn maturity_options(value: u32) -> Vec<String> {
    bit_labels(
        value,
        &[
            (1, "alcohol"),
            (2, "drugs"),
            (4, "violence"),
            (8, "explicit"),
        ],
    )
}

fn credit_options(value: u32) -> Vec<String> {
    bit_labels(
        value,
        &[
            (1, "show_credits"),
            (2, "original_or_permitted_assets"),
            (4, "redistribution_with_credit"),
            (8, "porting_with_credit"),
            (16, "patching_without_credit"),
            (32, "patching_with_credit"),
            (64, "patching_with_permission"),
            (128, "repackaging_without_credit"),
            (256, "repackaging_with_credit"),
            (512, "repackaging_with_permission"),
            (1024, "resale_allowed"),
        ],
    )
}

fn bit_labels(value: u32, labels: &[(u32, &str)]) -> Vec<String> {
    labels
        .iter()
        .filter(|(bit, _)| value & bit != 0)
        .map(|(_, label)| (*label).to_string())
        .collect()
}

fn attribution() -> ModIoAttribution {
    ModIoAttribution {
        name: "mod.io".to_string(),
        url: "https://mod.io/g/baldursgate3".to_string(),
        terms_url: "https://mod.io/terms".to_string(),
    }
}
