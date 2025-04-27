use serde::Serialize;

#[derive(Serialize)]
struct Endpoint {
    method: String,
    path: String,
    description: String,
}

#[derive(Serialize)]
struct DocsData {
    title: String,
    description: String,
    endpoints: Vec<Endpoint>,
}

#[derive(Debug, Serialize)]
pub struct ActionResult<T, E> {
    pub result: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

// Implementasi Default
impl<T, E> Default for ActionResult<T, E> {
    fn default() -> Self {
        Self {
            result: false, // Default-nya false
            message: String::new(),
            data: None,
            error: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Company {
    pub company_id: String,
    pub company_name: String,
}