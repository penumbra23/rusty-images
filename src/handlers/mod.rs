

use warp::{multipart::{FormData, Part}, Reply, Rejection, reply::{self, Response}, hyper::StatusCode, reject::{self, Reject}, http::HeaderValue};
use futures::TryStreamExt;
use bytes::BufMut;



use crate::image::{ImageResizeQuery, Image, ImageError, ImageFilter, OutputFormat};

use self::models::ImageStats;
pub mod models;

impl Reject for ImageError {}

pub async fn stats_handler(form: FormData) -> Result<impl Reply, Rejection> {
    let img = read_image(form).await?;
    let stats = ImageStats::new(img.img_data().width(), img.img_data().height(), img.size(), img.format().to_string());
    Ok(reply::with_status(reply::json(&stats), StatusCode::OK))
}

pub async fn resize_handler(width: u32, height: u32, form: FormData, params: ImageResizeQuery) -> Result<impl Reply, Rejection> {
    let img = read_image(form).await?;

    let filter = match params.filter_type {
        Some(f) => ImageFilter::parse(&f)?,
        None => ImageFilter::default(),
    };

    let resized_img = img.resize(width, height, filter, params.keep_aspect.unwrap());
    
    let format = match params.output_format {
        Some(f) => OutputFormat::parse(&f)?,
        None => OutputFormat::default(),
    };

    let mut data = Vec::new();
    resized_img.write_to(&mut data, format)?;

    let mut res = Response::new(data.into());
    res.headers_mut().insert("Content-Type", HeaderValue::from_str(img.format()).unwrap());

    Ok(reply::with_status(res, StatusCode::OK))
}

async fn read_image(form: FormData) -> Result<Image, Rejection> {
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

    let image_result = Image::parse(&value, &format)?;

    Ok(image_result)
}