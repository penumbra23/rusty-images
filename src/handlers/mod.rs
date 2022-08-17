

use log::info;
use warp::{multipart::{FormData, Part}, Reply, Rejection, reply::{self, Response}, hyper::StatusCode, reject::{self, Reject}, http::HeaderValue};
use futures::TryStreamExt;
use bytes::BufMut;

use crate::image::{ImageResizeQuery, Image, ImageError, ImageFilter, OutputFormat, ImageOutputQuery};

use self::models::ImageStats;
pub mod models;

impl Reject for ImageError {}

pub async fn stats_handler(form: FormData) -> Result<impl Reply, Rejection> {
    info!("Image stats");
    
    let img = read_image(form).await?;
    let stats = ImageStats::new(img.img_data().width(), img.img_data().height(), img.size(), img.format().to_string());
    Ok(reply::with_status(reply::json(&stats), StatusCode::OK))
}

pub async fn resize_handler(width: u32, height: u32, form: FormData, params: ImageResizeQuery) -> Result<impl Reply, Rejection> {
    info!("Image resize: w({}), h({}), params({:?})", width, height, params);

    let img = read_image(form).await?;
    
    let filter = match params.filter_type {
        Some(f) => ImageFilter::parse(&f)?,
        None => ImageFilter::default(),
    };

    let format = match params.output_format {
        Some(f) => OutputFormat::parse(&f)?,
        None => OutputFormat::default(),
    };

    let resized_img = img.resize(width, height, filter, params.keep_aspect.unwrap());
    
    let mut data = Vec::new();
    resized_img.write_to(&mut data, &format)?;

    let mut res = Response::new(data.into());
    let content_type = format!("image/{}", <OutputFormat as Into<String>>::into(format));
    res.headers_mut().insert("Content-Type", HeaderValue::from_str(content_type.as_str()).unwrap());

    Ok(reply::with_status(res, StatusCode::OK))
}

pub async fn blur_handler(strength: f32, form: FormData, output_params: ImageOutputQuery) -> Result<impl Reply, Rejection> {
    info!("Image blur: strength({})", strength);

    if strength < 0.0 {
        return Err(reject::custom(
                ImageError::InvalidFormat(format!("strength param should be positive; supplied {}", strength))));
    }

    let img = read_image(form).await?;

    let format = match output_params.output_format {
        Some(f) => OutputFormat::parse(&f)?,
        None => OutputFormat::default(),
    };

    let blurred_img = img.blur(strength);
    
    let mut data = Vec::new();
    blurred_img.write_to(&mut data, &format)?;

    let mut res = Response::new(data.into());
    let content_type = format!("image/{}", <OutputFormat as Into<String>>::into(format));
    res.headers_mut().insert("Content-Type", HeaderValue::from_str(content_type.as_str()).unwrap());

    Ok(reply::with_status(res, StatusCode::OK))
}

async fn read_image(form: FormData) -> Result<Image, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|err| {
        reject::custom(ImageError::InvalidFormat(format!("read error: {}", err)))
    })?;
    
    let file = match parts.into_iter().find(|p| p.name() == "file") {
        Some(file) => file,
        None => {
            return Err(reject::custom(ImageError::InvalidFormat(String::from("file not found in request"))))
        },
    };

    let content_type = file.content_type().map(|s| s.to_owned());
    let format = match content_type {
        Some(file_type) => file_type,
        None => {
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
        .map_err(|err| {
            reject::custom(ImageError::ReadError(format!("{}", err)))
        })?;

    let image_result = Image::parse(&value, &format)?;

    Ok(image_result)
}