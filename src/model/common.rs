use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponse<T> {
    pub content: Vec<T>,
    pub page: i32,
    pub size: i32,
    pub total_elements: i64,
    pub total_pages: i32,
    pub is_first: bool,
    pub is_last: bool,
}

impl<T> PaginationResponse<T> {
    pub fn new(content: Vec<T>, page: i32, size: i32, total_elements: i64) -> Self {
        let total_pages = (total_elements as f64 / size as f64).ceil() as i32;
        let is_first = page == 1;
        let is_last = page >= total_pages;

        Self {
            content,
            page,
            size,
            total_elements,
            total_pages,
            is_first,
            is_last,
        }
    }
}