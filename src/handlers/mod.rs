use std::{io::Cursor, fmt::Display};

use warp::{multipart::{FormData, Part}, Reply, Rejection, reply, hyper::StatusCode, reject::{Reject, self}};
use futures::TryStreamExt;

use bytes::BufMut;
use image::{io::Reader as ImageReader, DynamicImage};

use self::models::ImageStats;
pub mod models;

#[derive(Clone, Debug)]
pub enum ImageError {
    InvalidFormat(String),
    ReadError(String),
}

impl Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "image err: {:?}", self)
    }
}

impl Reject for ImageError {}

struct ImageReadResult {
    img_data: DynamicImage,
    size: usize,
    format: String,
}

pub async fn stats_handler(form: FormData) -> Result<impl Reply, Rejection> {
    let img = read_image(form).await?;
    let stats = ImageStats::new(img.img_data.width(), img.img_data.height(), img.size,img.format);
    Ok(reply::with_status(reply::json(&stats), StatusCode::OK))
}

async fn read_image(form: FormData) -> Result<ImageReadResult, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        reject::custom(ImageError::InvalidFormat(e.to_string()))
    })?;
    
    let file = match parts.into_iter().find(|p| p.name() == "file") {
        Some(file) => file,
        None => {
            eprintln!("file not found");
            return Err(reject::custom(ImageError::InvalidFormat(String::from("file not found in request"))))
        },
    };

    let content_type = file.content_type().map(|s| s.to_owned());
    let format = match content_type {
        Some(file_type) => file_type,
        None => {
            eprintln!("file type could not be determined");
            return Err(reject::custom(ImageError::InvalidFormat(String::from("file type could not be determined"))));
        }
    };

    let value = file
        .stream()
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await
        .map_err(|e| {
            eprintln!("reading file error: {}", e);
            reject::custom(ImageError::ReadError(String::from("error reading image")))
        })?;

    let img_size = value.len();

    let img = ImageReader::new(Cursor::new(value))
        .with_guessed_format()
        .map_err(|err| {
            eprintln!("reading image error: {}", err);
            reject::custom(ImageError::ReadError(String::from("error reading image")))
        })?
        .decode()
        .map_err(|err| {
            eprintln!("decoding image error: {}", err);
            reject::custom(ImageError::ReadError(String::from("error decoding image")))
        })?;

    Ok(ImageReadResult {
        format: format.to_string(),
        img_data: img,
        size: img_size,
    })
}