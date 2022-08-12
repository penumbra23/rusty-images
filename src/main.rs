mod handlers;

use std::convert::Infallible;

use handlers::{models::ErrorMessage, stats_handler};
use warp::{Filter, Reply, Rejection, reply, hyper::StatusCode, reject};


async fn handle_reject(rej: Rejection) -> Result<impl Reply, Infallible> {
    println!("{:?}", rej);

    let message;
    let status_code;

    if rej.is_not_found() {
        status_code = StatusCode::NOT_FOUND;
        message = "Resource not found".to_string();
    } else if let Some(_) = rej.find::<reject::MethodNotAllowed>() {
        status_code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method now allowed".to_string();
    } else if let Some(header_err) = rej.find::<reject::InvalidHeader>() {
        status_code = StatusCode::BAD_REQUEST;
        message = format!("Bad request: {}", header_err);
    } else {
        // TODO: log
        status_code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Error on server side".to_string();
    }

    Ok(reply::with_status(reply::json(&ErrorMessage::new(&message)), status_code))
}

#[tokio::main]
async fn main() {
    let stats_route = warp::path!("stats")
        .and(warp::post())
        .and(warp::multipart::form().max_length(10_000_000))
        .and_then(stats_handler);

    let router = stats_route.recover(handle_reject);

    warp::serve(router)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


