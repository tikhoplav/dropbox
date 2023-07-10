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
use std::io::ErrorKind;
use futures_util::{
    TryStreamExt,
    StreamExt,
};

#[tokio::main]
async fn main() {    
    let status = warp::get()
        .and(warp::path::end())
        .and_then(handle_status);

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
        Err(err) => {
            let msg = format!("Failed to create a folder: {}\n", err);
            return Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR));
        }
    };

    let mut parts = form.into_stream();
    while let Some(next) = parts.next().await {
        match next {
            Ok(part) => {
                let filename = part.name();
                let filepath = format!("{}/{}", path, filename);

                let data = match part
                    .stream()
                    .try_fold(Vec::new(), |mut acc, buf| async move {
                        acc.extend_from_slice(buf.chunk());
                        Ok(acc)
                    })
                    .await
                {
                    Ok(data) => data,
                    Err(err) => {
                        let msg = format!("Failed to read data from a part: {}\n", err.to_string());
                        return Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR));
                    }
                };

                match std::fs::write(filepath, data) {
                    Ok(()) => (),
                    Err(err) => {
                        let msg = format!("Failed to write file: {}\n", err.to_string());
                        return Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR));
                    }
                };
            },
            Err(err) => {
                let msg = format!("Failed to read a part: {}\n", err.to_string());
                return Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR));
            },
        }
    }

    Ok(warp::reply::with_status("ok\n".to_string(), StatusCode::OK))
}

async fn handle_put(_path: FullPath, _form: FormData) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("not implemented\n", StatusCode::NOT_IMPLEMENTED))
}

async fn handle_patch(_path: FullPath, _form: FormData) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("not implemented\n", StatusCode::NOT_IMPLEMENTED))
}

async fn handle_remove(path: FullPath) -> Result<impl Reply, Rejection> {
    let filepath = format!("/data/{}", path.as_str());
    match std::fs::remove_file(filepath) {
        Ok(()) => Ok(warp::reply::with_status("ok\n".to_string(), StatusCode::NO_CONTENT)),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            Ok(warp::reply::with_status("not found\n".to_string(), StatusCode::NOT_FOUND))
        },
        Err(err) => {
            let msg = format!("Failed to remove file: {}\n", err.to_string());
            Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
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