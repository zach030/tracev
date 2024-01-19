use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::path::Path;
use regex::Regex;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use atty::Stream;

#[tokio::main]
async fn main() {
    if atty::is(Stream::Stdin) {
        println!("Usage: forge test -vvvv --mt test_function | tracev");
        return;
    }
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut buffer = String::new();
    let re = Regex::new("\x1b\\[[0-9;]*m").unwrap();  

    while let Some(line) = lines.next_line().await.unwrap() {
        println!("{}", &line); 
        let cleaned_line = re.replace_all(&line, "");
        buffer.push_str(&cleaned_line);
        buffer.push('\n');
    }

    start_server(&buffer).await;
}

async fn start_server(trace_text: &str) {
    let addr = ([127, 0, 0, 1], 3000).into();
    let make_svc = make_service_fn(|_conn| {
        let trace_text = trace_text.to_string();
        async {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                serve_req(trace_text.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


async fn serve_req(trace_text: String) -> Result<Response<Body>, hyper::Error> {
    let html_dir = std::env::var("HOME").unwrap() + "/.tracev/templates";
    let html_path = Path::new(&html_dir).join("index.html");

    let mut file = File::open(html_path).await.unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.unwrap();

    let html = contents.replace("{{ trace_text | safe }}", &trace_text);
    Ok(Response::new(Body::from(html)))
}
