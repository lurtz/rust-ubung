// redo html example

use std::fmt::Display;

pub trait ResponseState {}
pub trait SendingState {}
struct Start {}
impl ResponseState for Start {}
struct Headers {
    status_line: (u8, String),
    header: Vec<(String, String)>,
}
impl ResponseState for Headers {}
impl SendingState for Headers {}
struct Body {
    headers: Headers,
    body: String,
}
impl ResponseState for Body {}
impl SendingState for Body {}

pub struct HttpResponse<S: ResponseState> {
    _sending_state: S,
}

impl<S> HttpResponse<S>
where
    S: ResponseState + SendingState,
{
    pub fn send(self) {}
}

impl Display for HttpResponse<Body> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {}",
            self._sending_state.headers.status_line,
            self._sending_state.headers.header,
            self._sending_state.body
        )
    }
}

impl Default for HttpResponse<Start> {
    fn default() -> Self {
        Self {
            _sending_state: Start {},
        }
    }
}

impl HttpResponse<Start> {
    pub fn status_line(self, code: u8, message: &str) -> HttpResponse<Headers> {
        HttpResponse {
            _sending_state: Headers {
                status_line: (code, message.to_string()),
                header: Vec::new(),
            },
        }
    }
}

impl HttpResponse<Headers> {
    pub fn header(mut self, key: &str, value: &str) -> HttpResponse<Headers> {
        self._sending_state
            .header
            .push((key.to_string(), value.to_string()));
        HttpResponse {
            _sending_state: self._sending_state,
        }
    }

    pub fn body(self, body: &str) -> HttpResponse<Body> {
        HttpResponse {
            _sending_state: Body {
                headers: self._sending_state,
                body: body.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::HttpResponse;

    #[test]
    fn create_valid_response() {
        let httpresponse = HttpResponse::default();
        httpresponse
            .status_line(123, "blub")
            .header("Length", "123")
            .header("Spam-value", "666")
            .body("aaaaaaaahhhhh");
    }

    #[test]
    fn check_display() {
        let httpresponse = HttpResponse::default();
        let body = httpresponse
            .status_line(123, "blub")
            .header("Length", "123")
            .header("Spam-value", "666")
            .body("aaaaaaaahhhhh");

        assert_eq!(
            "(123, \"blub\") [(\"Length\", \"123\"), (\"Spam-value\", \"666\")] aaaaaaaahhhhh",
            format!("{}", body)
        );
    }

    #[test]
    fn send_with_body() {
        let httpresponse = HttpResponse::default();
        httpresponse
            .status_line(123, "blub")
            .header("Length", "123")
            .header("Spam-value", "666")
            .body("aaaaaaaahhhhh")
            .send();
    }

    #[test]
    fn send_with_status() {
        let httpresponse = HttpResponse::default();
        httpresponse.status_line(123, "blub").send();
    }
}
