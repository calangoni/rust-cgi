use std::env;
use std::str::FromStr;
use std::io::{Read, Write, stdin, stdout};

fn main() {
    let request = match parse_cgi_req() {
        Ok(v) => v,
        Err(err) => {
            let response = respond_cgi_plain_text(&err);
            stdout().write_all(build_header(&response).as_bytes()).unwrap();
            stdout().write_all(&response.content).unwrap();
            return;
        }
    };

    let response = match process_request(request) {
        Ok(v) => v,
        Err(err) => respond_cgi_plain_text(&err),
    };
  
    stdout().write_all(build_header(&response).as_bytes()).unwrap();
    stdout().write_all(&response.content).unwrap();
}

fn process_request(req: CgiRequest) -> Result<CgiResponse, String> {
    let mut response = json::JsonValue::new_object();
    if (req.content.len() > 0) {
        let body_str = String::from_utf8_lossy(&req.content);
        let parsed = match json::parse(&body_str) { Ok(v) => v, Err(_) => return Err("Error parsing body".to_owned()) };
        // let prop = match parsed["prop"].as_str() { Some(v) => v, None => { return Err("Missing prop".to_owned()); } };
        response["req_body"] = parsed.into();
    } else {
        response["req_body"] = "".into();
    }
    return Ok(respond_cgi_json(&response));
}

fn parse_cgi_req () -> Result<CgiRequest, String> {
    let content_length = match env::var("CONTENT_LENGTH") {
        Err(_) => 0,
        Ok(v) => match usize::from_str(&v) {
            Err(_) => { return Err("Invalid CONTENT_LENGTH".to_owned()); },
            Ok(v) => v,
        },
    };

    let method = match env::var("REQUEST_METHOD") {
        Err(_) => { return Err("No REQUEST_METHOD".to_owned()); },
        Ok(v) => v,
    };

    let path = env::var("PATH_INFO").unwrap_or("/".to_owned());
    // CONTEXT_DOCUMENT_ROOT="/home/http/public_html/cgi-bin/"
    // CONTEXT_PREFIX="/cgi-bin/"
    // DOCUMENT_ROOT="/home/http/public_html"
    // GATEWAY_INTERFACE="CGI/1.1"
    // HTTP_ACCEPT="text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
    // HTTP_ACCEPT_ENCODING="gzip, deflate"
    // HTTP_CONNECTION="keep-alive"
    // HTTP_DNT="1"
    // HTTP_HOST="localhost"
    // HTTP_UPGRADE_INSECURE_REQUESTS="1"
    // HTTP_USER_AGENT="Mozilla/5.0 (X11; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0"
    // PATH_INFO="/foo/bar"
    // PATH_TRANSLATED="/home/http/public_html/foo/bar"
    // QUERY_STRING="var1=value1&var2=with%20percent%20encoding"
    // REMOTE_ADDR="127.0.0.1"
    // REMOTE_PORT="35626"
    // REQUEST_METHOD="GET"
    // REQUEST_SCHEME="http"
    // REQUEST_URI="/cgi-bin/emp/foo/bar?var1=value1&var2=with%20percent%20encoding"
    // SCRIPT_FILENAME="/home/http/public_html/cgi-bin/emp"
    // SCRIPT_NAME="/cgi-bin/emp"
    // SERVER_ADDR="127.0.0.1"
    // SERVER_NAME="localhost"
    // SERVER_PORT="80"
    // SERVER_PROTOCOL="HTTP/1.1"
    // SERVER_SIGNATURE=""
    // SERVER_SOFTWARE="Apache/2.4.46 (Unix)"
    // AUTH_TYPE
    // CONTENT_LENGTH
    // CONTENT_TYPE
    // REMOTE_HOST
    // REMOTE_IDENT
    // REMOTE_USER

    let mut content = vec![0; content_length];
    match stdin().read_exact(&mut content) {
        Err(_) => { return Err("Could not read stdin".to_owned()); },
        Ok(_) => (),
    };

    return Ok(CgiRequest{
        method: method,
        path: path,
        content: content,
    })
}

fn build_header (response: &CgiResponse) -> String {
    let mut header = String::with_capacity(200);
    for i in 0..response.headers.len() {
      header += &format!("{}: {}\r\n", &response.headers[i].attribute, &response.headers[i].value);
    }
    header += "\r\n";
    return header;
}

fn respond_cgi_json (content: &json::JsonValue) -> CgiResponse {
    let content_bytes = content.dump().as_bytes().to_vec();
    return CgiResponse{
        headers: vec![
            HttpHeaderEntry{ attribute: "Content-Type".to_owned(), value: "application/json; charset=UTF-8".to_owned() },
            HttpHeaderEntry{ attribute: "Content-Length".to_owned(), value: content_bytes.len().to_string() },
        ],
        content: content_bytes,
    };
}

fn respond_cgi_plain_text (content: &str) -> CgiResponse {
    let content_bytes = content.as_bytes().to_vec();
    return CgiResponse{
        headers: vec![
            HttpHeaderEntry{ attribute: "Content-Type".to_owned(), value: "text/plain; charset=UTF-8".to_owned() },
            HttpHeaderEntry{ attribute: "Content-Length".to_owned(), value: content_bytes.len().to_string() },
        ],
        content: content_bytes,
    };
}

pub struct HttpHeaderEntry {
    pub attribute: String,
    pub value: String,
}

pub struct CgiRequest {
    pub method: String,
    pub path: String,
    pub content: Vec::<u8>,
}

pub struct CgiResponse {
    pub headers: Vec::<HttpHeaderEntry>,
    pub content: Vec::<u8>,
}
