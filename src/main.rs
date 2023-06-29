use warp::{
    Filter,
    Rejection,
    Reply,
    path::FullPath,
    http::StatusCode,
    multipart::{
        FormData,
    },
};
use std::convert::Infallible;

#[tokio::main]
async fn main() {
    let serve = warp::get().and(warp::fs::dir("/data"));

    let upload = warp::post()
        .and(warp::path::full())
        .and(warp::multipart::form())
        .and_then(handle_form);
        
    let remove = warp::delete()
        .and(warp::path::full())
        .and_then(handle_remove);

    let router = serve.or(upload).or(remove).recover(handle_reject);
    warp::serve(router).run(([0, 0, 0, 0], 80)).await;
}

async fn handle_form(path: FullPath, _form: FormData) -> Result<impl Reply, Rejection> {
    println!("{:?}", path);
    Ok("ok")
}

async fn handle_remove(path: FullPath) -> Result<impl Reply, Rejection> {
    println!("Removing file with path {:?}", path);
    Ok("ok")
}

async fn handle_reject(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "not found".to_string())
    } else {
        eprintln!("{:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "server error".to_string())
    };

    Ok(warp::reply::with_status(message, code))
}