// redo html example

#[cfg(test)]
mod test {
    use std::fmt::Display;

    trait SendingState {}

    struct Start {}
    impl SendingState for Start {}

    struct Headers {
        status_line: (u8, String),
        header: Vec<(String, String)>,
    }
    impl SendingState for Headers {}

    struct Body {
        headers: Headers,
        body: String,
    }
    impl SendingState for Body {}

    struct HttpResponse<S: SendingState> {
        _sending_state: S,
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

    impl HttpResponse<Start> {
        fn new() -> HttpResponse<Start> {
            HttpResponse::<Start> {
                _sending_state: Start {},
            }
        }

        fn status_line(self, code: u8, message: &str) -> HttpResponse<Headers> {
            HttpResponse {
                _sending_state: Headers {
                    status_line: (code, message.to_string()),
                    header: Vec::new(),
                },
            }
        }
    }

    impl HttpResponse<Headers> {
        fn header(mut self, key: &str, value: &str) -> HttpResponse<Headers> {
            self._sending_state
                .header
                .push((key.to_string(), value.to_string()));
            HttpResponse {
                _sending_state: self._sending_state,
            }
        }

        fn body(self, body: &str) -> HttpResponse<Body> {
            HttpResponse {
                _sending_state: Body {
                    headers: self._sending_state,
                    body: body.to_string(),
                },
            }
        }
    }

    impl HttpResponse<Body> {}

    #[test]
    fn create_valid_response() {
        let httpresponse = HttpResponse::new();
        httpresponse
            .status_line(123, "blub")
            .header("Length", "123")
            .header("Spam-value", "666")
            .body("aaaaaaaahhhhh");
    }

    #[test]
    fn check_display() {
        let httpresponse = HttpResponse::new();
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
}
