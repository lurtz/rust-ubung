// redo html example

#[cfg(test)]
mod test {
    use std::{fmt::Display, marker::PhantomData};

    struct ActualResponseState {
        status_line: (u8, String),
        header: Vec<(String, String)>,
        body: String,
    }

    impl ActualResponseState {
        fn new() -> ActualResponseState {
            ActualResponseState {
                status_line: (0u8, String::new()),
                header: Vec::new(),
                body: String::new(),
            }
        }
    }

    trait SendingState {}

    struct Start {}
    impl SendingState for Start {}

    struct Headers {}
    impl SendingState for Headers {}

    struct Body {}
    impl SendingState for Body {}

    struct HttpResponse<S: SendingState> {
        state: Box<ActualResponseState>,
        _sending_state: PhantomData<S>,
    }

    impl<S: SendingState> Display for HttpResponse<S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{:?} {:?} {}",
                self.state.status_line, self.state.header, self.state.body
            )
        }
    }

    impl HttpResponse<Start> {
        fn new() -> HttpResponse<Start> {
            HttpResponse::<Start> {
                state: Box::new(ActualResponseState::new()),
                _sending_state: PhantomData::default(),
            }
        }

        fn status_line(mut self, code: u8, message: &str) -> HttpResponse<Headers> {
            self.state.status_line = (code, message.to_string());
            HttpResponse {
                state: self.state,
                _sending_state: PhantomData::default(),
            }
        }
    }

    impl HttpResponse<Headers> {
        fn header(mut self, key: &str, value: &str) -> HttpResponse<Headers> {
            self.state.header.push((key.to_string(), value.to_string()));
            HttpResponse {
                state: self.state,
                _sending_state: PhantomData::default(),
            }
        }

        fn body(mut self, body: &str) -> HttpResponse<Body> {
            self.state.body = body.to_string();
            HttpResponse {
                state: self.state,
                _sending_state: PhantomData::default(),
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
