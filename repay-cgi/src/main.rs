fn main() {
    cgi::handle(|request: cgi::Request| -> cgi::Response {
        let input = match String::from_utf8(request.into_body()) {
            Ok(s) => s,
            Err(_) => return cgi::empty_response(400),
        };
        let result = repay::run(input.lines().map(str::to_owned));
        let response_body: String = result.iter().map(|debt| format!("{}\n", debt)).collect();
        cgi::string_response(200, response_body)
    })
}
