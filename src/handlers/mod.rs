use warp::{multipart::FormData, Reply, Rejection, reply, hyper::StatusCode};

use self::models::ImageStats;

pub mod models;

pub async fn stats_handler(form: FormData) -> Result<impl Reply, Rejection> {
    Ok(reply::with_status(reply::json(&ImageStats::new(2)), StatusCode::OK))
}