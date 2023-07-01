use warp::{
    Buf,
    Filter,
    Rejection,
    Reply,
    path::FullPath,
    http::{
        StatusCode,
        header::{
            HeaderMap,
            HeaderValue,
        },
    },
    multipart::{
        FormData,
    },
    reject::{
        MethodNotAllowed,
        PayloadTooLarge,
        LengthRequired,
        InvalidHeader,
    }
};
use std::convert::Infallible;
use futures_util::{
    TryStreamExt,
    StreamExt,
};

#[tokio::main]
async fn main() {    
    let status = warp::path::end().and_then(handle_status);
    let options = warp::options().and_then(handle_status);

    let upload = warp::post()
        .and(warp::path::full())
        .and(warp::multipart::form().max_length(20_000_000))
        .and_then(handle_upload);

    let put = warp::put()
        .and(warp::path::full())
        .and(warp::multipart::form().max_length(20_000_000))
        .and_then(handle_put);

    let patch = warp::patch()
        .and(warp::path::full())
        .and(warp::multipart::form().max_length(20_000_000))
        .and_then(handle_patch);

    let remove = warp::delete()
        .and(warp::path::full())
        .and_then(handle_remove);

    let serve = warp::get().and(warp::fs::dir("/data"));

    let mut headers = HeaderMap::new();
    headers.insert(
        "Allow",
        HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, OPTIONS")
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, OPTIONS")
    );
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type"));

    let router = options
        .or(status)
        .or(serve)
        .or(upload)
        .or(put)
        .or(patch)
        .or(remove)
        .with(warp::reply::with::headers(headers))
        .recover(handle_reject);

    warp::serve(router).run(([0, 0, 0, 0], 80)).await;
}

async fn handle_status() -> Result<impl Reply, Rejection> {
    Ok("ok")
}

async fn handle_upload(path_raw: FullPath, form: FormData) -> Result<impl Reply, Rejection> {
    let mut path = path_raw.as_str().to_string();
    if path.ends_with("/") {
        path = (&path[..path.len() - 1]).to_string();
    }
    path = format!("/data/{}", path);

    match std::fs::create_dir_all(path.clone()) {
        Ok(()) => {},
        Err(err) => panic!("{}", err),
    };

    let mut parts = form.into_stream();
    while let Some(next) = parts.next().await {
        match next {
            Ok(part) => {
                let filename = part.name();
                let filepath = format!("{}/{}", path, filename);

                let data = part
                    .stream()
                    .try_fold(Vec::new(), |mut acc, buf| async move {
                        acc.extend_from_slice(buf.chunk());
                        Ok(acc)
                    })
                    .await.expect("folding error");

                match std::fs::write(filepath, data) {
                    Ok(()) => (),
                    Err(err) => panic!("{}", err),
                };
            },
            Err(err) => {
                panic!("{}", err);
            },
        }
    }

    Ok(warp::reply::with_status("ok", StatusCode::CREATED))
}

async fn handle_put(_path: FullPath, _form: FormData) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("not implemented", StatusCode::NOT_IMPLEMENTED))
}

async fn handle_patch(_path: FullPath, _form: FormData) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("not implemented", StatusCode::NOT_IMPLEMENTED))
}

async fn handle_remove(_path: FullPath) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("not implemented", StatusCode::NOT_IMPLEMENTED))
}

async fn handle_reject(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "not found".to_string())
    } else if let Some(e) = err.find::<InvalidHeader>() {
        (StatusCode::BAD_REQUEST, format!("{:?}", e))
    } else if err.find::<LengthRequired>().is_some() {
        (StatusCode::BAD_REQUEST, "length is not set".to_string())
    } else if err.find::<PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "body is too big".to_string())
    } else if err.find::<MethodNotAllowed>().is_some() {
        // This is necessary, because MethodNotAllowed has a higher
        // priority over NotFound, and the previous statement never
        // turns true, when warp::fs is used with method filters.
        (StatusCode::NOT_FOUND, "not found".to_string())
    } else {
        eprintln!("{:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "server error".to_string())
    };

    Ok(warp::reply::with_status(message, code))
}