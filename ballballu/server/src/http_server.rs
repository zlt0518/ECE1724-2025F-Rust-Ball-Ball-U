use std::convert::Infallible;
use std::path::PathBuf;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::fs;

/// Simple HTTP server to serve static files
pub struct HttpServer {
    pub addr: String,
    pub static_dir: PathBuf,
}

impl HttpServer {
    pub fn new(addr: &str, static_dir: PathBuf) -> Self {
        Self {
            addr: addr.to_string(),
            static_dir,
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(&self.addr).await.unwrap();
        println!("HTTP server running at http://{}/", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let io = TokioIo::new(stream);
                    let static_dir = self.static_dir.clone();

                    tokio::spawn(async move {
                        let service = service_fn(move |req| handle_request(req, static_dir.clone()));

                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {:?}", e);
                }
            }
        }
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    static_dir: PathBuf,
) -> Result<Response<http_body_util::Full<Bytes>>, Infallible> {
    let path = req.uri().path();

    // Default to index.html if root path
    let file_path = if path == "/" || path == "" {
        static_dir.join("test.html")
    } else {
        // Remove leading slash
        let clean_path = path.trim_start_matches('/');
        static_dir.join(clean_path)
    };

    // Security: prevent directory traversal
    if !file_path.starts_with(&static_dir) {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(http_body_util::Full::new(Bytes::from("Forbidden")))
            .unwrap());
    }

    match fs::read_to_string(&file_path).await {
        Ok(content) => {
            let content_type = get_content_type(&file_path);
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .body(http_body_util::Full::new(Bytes::from(content)))
                .unwrap();
            Ok(response)
        }
        Err(_) => {
            // If file not found, try test.html
            if file_path != static_dir.join("test.html") {
                match fs::read_to_string(static_dir.join("test.html")).await {
                    Ok(content) => {
                        let response = Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/html")
                            .body(http_body_util::Full::new(Bytes::from(content)))
                            .unwrap();
                        Ok(response)
                    }
                    Err(_) => {
                        let response = Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(http_body_util::Full::new(Bytes::from("File not found")))
                            .unwrap();
                        Ok(response)
                    }
                }
            } else {
                let response = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(http_body_util::Full::new(Bytes::from("File not found")))
                    .unwrap();
                Ok(response)
            }
        }
    }
}

fn get_content_type(path: &PathBuf) -> &str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "text/plain",
    }
}

