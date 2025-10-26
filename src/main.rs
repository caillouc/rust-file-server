// AI GENERATED CODE

use std::path::PathBuf;
use std::env;
use warp::Filter;
use warp::reply::{Reply, Response};
use warp::http::StatusCode;
use mime_guess;

#[tokio::main]
async fn main() {
    // Get the directory to serve files from environment variable or default to current directory
    let serve_dir: String = "./files".into();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .unwrap();

    println!("Starting file server...");
    println!("Serving files from: {}", serve_dir);
    println!("Server running on: http://0.0.0.0:{}", port);

    // Root route
    let root = warp::path::end().map(|| "Flash Backend File Server is running!");

    // Route to serve files - handle nested paths
    let files = warp::path::tail()
        .and_then(move |path: warp::path::Tail| {
            let serve_dir = serve_dir.clone();
            async move {
                serve_file(serve_dir, path.as_str().to_string()).await
            }
        });

    let routes = root.or(files)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

async fn serve_file(serve_dir: String, filename: String) -> Result<Response, warp::Rejection> {
    // Security: Normalize and validate the requested path
    let requested_path = PathBuf::from(&filename);
    
    // Security: Reject paths with ".." components to prevent directory traversal
    if requested_path.components().any(|comp| matches!(comp, std::path::Component::ParentDir)) {
        return Ok(warp::reply::with_status("Access denied", StatusCode::FORBIDDEN).into_response());
    }
    
    // Security: Reject absolute paths
    if requested_path.is_absolute() {
        return Ok(warp::reply::with_status("Access denied", StatusCode::FORBIDDEN).into_response());
    }
    
    let base_dir = PathBuf::from(&serve_dir).canonicalize().map_err(|_| {
        warp::reject::not_found()
    })?;
    
    let file_path = base_dir.join(&requested_path);
    
    // Security: Ensure the resolved path is still within the base directory
    let canonical_file_path = match file_path.canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(warp::reply::with_status("File not found", StatusCode::NOT_FOUND).into_response()),
    };
    
    if !canonical_file_path.starts_with(&base_dir) {
        return Ok(warp::reply::with_status("Access denied", StatusCode::FORBIDDEN).into_response());
    }
    
    // Check if file exists
    if !canonical_file_path.exists() {
        return Ok(warp::reply::with_status("File not found", StatusCode::NOT_FOUND).into_response());
    }

    // Check if it's a file (not a directory)
    if !canonical_file_path.is_file() {
        return Ok(warp::reply::with_status("Not a file", StatusCode::BAD_REQUEST).into_response());
    }

    // Read the file content
    match tokio::fs::read(&canonical_file_path).await {
        Ok(contents) => {
            // Guess the MIME type based on file extension
            let mime_type = mime_guess::from_path(&canonical_file_path)
                .first_or_octet_stream()
                .to_string();

            // Create response with proper content type
            let mut response = Response::new(contents.into());
            response.headers_mut().insert(
                "content-type",
                mime_type.parse().unwrap()
            );
            Ok(response)
        },
        Err(_) => Ok(warp::reply::with_status("Internal server error", StatusCode::INTERNAL_SERVER_ERROR).into_response()),
    }
}
