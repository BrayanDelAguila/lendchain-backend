use serde::{Deserialize, Serialize};

/// Generic paginated response wrapper.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Pagination query parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        (self.page.max(1) - 1) * self.per_page.clamp(1, 100)
    }

    pub fn limit(&self) -> i64 {
        self.per_page.clamp(1, 100)
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        let per_page = params.limit();
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            data,
            total,
            page: params.page,
            per_page,
            total_pages,
        }
    }
}
