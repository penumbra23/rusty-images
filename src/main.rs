mod handlers;
mod image;

use std::convert::Infallible;

use log::error;
use serde::de::DeserializeOwned;
use warp::{Filter, Reply, Rejection, reply, hyper::StatusCode, reject, filters::BoxedFilter};

use crate::image::{ImageError, ImageResizeQuery};
use crate::handlers::{models::ErrorMessage, stats_handler, resize_handler};

async fn handle_reject(err: Rejection) -> Result<impl Reply, Infallible> {
    error!("{:?}", err);

    let message;
    let status_code;

    if err.is_not_found() {
        status_code = StatusCode::NOT_FOUND;
        message = "Resource not found".to_string();
    } else if err.find::<reject::MethodNotAllowed>().is_some() {
        status_code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method now allowed".to_string();
    } else if let Some(header_err) = err.find::<reject::InvalidHeader>() {
        status_code = StatusCode::BAD_REQUEST;
        message = format!("Bad request: {}", header_err);
    } else if let Some(payload_err) = err.find::<reject::PayloadTooLarge>() {
        status_code = StatusCode::BAD_REQUEST;
        message = format!("Bad request: {}", payload_err);
    } else if let Some(image_err) = err.find::<ImageError>() {
        status_code = StatusCode::BAD_REQUEST;
        message = image_err.to_string();
    } else {
        status_code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Error on server side".to_string();
    }

    Ok(reply::with_status(reply::json(&ErrorMessage::new(&message)), status_code))
}

fn optional_query<T: 'static+Default+Send+DeserializeOwned>() -> BoxedFilter<(T,)> {
    warp::any()
        .and(warp::query().or(warp::any().map(|| T::default())))
        .unify()
        .boxed()
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stats_route = warp::path!("stats")
        .and(warp::post())
        .and(warp::multipart::form().max_length(10_000_000))
        .and_then(stats_handler);

    let resize_route = warp::path!("resize" / u32 / u32)
        .and(warp::post())
        .and(warp::multipart::form().max_length(10_000_000))
        .and(optional_query::<ImageResizeQuery>())
        .and_then(resize_handler);

    let log = warp::log::custom(|info| {
        log::info!("{} {} {} {:?}", info.method(), info.path(), info.status(), info.remote_addr());
    });

    let router = stats_route
        .or(resize_route)
        .recover(handle_reject)
        .with(log);
    
    warp::serve(router)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


