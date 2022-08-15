use std::{fmt::Display, io::Cursor};

use image::{io::Reader as ImageReader, DynamicImage, ImageOutputFormat, imageops::FilterType};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub enum ImageError {
    InvalidFormat(String),
    ReadError(String),
    ResizeError(String),
}

impl Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "image err: {:?}", self)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImageOutputQuery {
    pub output_format: Option<String>,
}

impl Default for ImageOutputQuery {
    fn default() -> Self {
        Self { output_format: Some(String::from("png")) }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImageResizeQuery {
    pub keep_aspect: Option<bool>,
    pub output_format: Option<String>,
    pub filter_type: Option<String>,
}

impl Default for ImageResizeQuery {
    fn default() -> Self {
        Self { keep_aspect: Some(true), output_format: Some(String::from("png")), filter_type: Some(String::from("nearest")) }
    }
}

pub struct Image {
    img_data: DynamicImage,
    size: usize,
    format: String,
}

impl Image {
    pub fn parse(bytes: &Vec<u8>, format: &str) -> Result<Image, ImageError> {
        let dyn_img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| {
            ImageError::ReadError(format!("error reading image: {}", err))
        })?
        .decode()
        .map_err(|err| {
            ImageError::ReadError(format!("error reading image: {}", err))
        })?;
        
        Ok(Image {
            format: format.to_string(),
            img_data: dyn_img,
            size: bytes.len(),
        })
    }

    pub fn img_data(&self) -> &DynamicImage {
        &self.img_data
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn resize(&self, width: u32, height: u32, filter: ImageFilter, keep_aspect: bool) -> Image {
        let resized_img = match keep_aspect {
            true => self.img_data.resize(width, height, filter.filter),
            false => self.img_data.resize_exact(width, height, filter.filter),
        };
        let size = resized_img.as_bytes().len();
        Image { img_data: resized_img, size, format: self.format.clone() }
    }

    pub fn blur(&self, strength: f32) -> Image {
        let blurred_img = self.img_data.blur(strength);
        Image { img_data: blurred_img, size: self.size, format: self.format.clone() }
    }

    pub fn write_to(&self, buf: &mut Vec<u8>, format: OutputFormat) -> Result<(), ImageError> {
        self.img_data.write_to(&mut Cursor::new(buf), format.format)
        .map_err(|err| {
            ImageError::ResizeError(format!("image write error: {}", err))
        })?;
        Ok(())
    }
}

pub struct ImageFilter {
    filter: FilterType,
}

impl ImageFilter {
    pub fn parse(str: &str) -> Result<ImageFilter, ImageError> {
        match str.to_lowercase().as_str() {
            "nearest" => Ok(ImageFilter{filter: FilterType::Nearest}),
            "gaussian" => Ok(ImageFilter{filter: FilterType::Gaussian}),
            _ => Err(ImageError::ResizeError(format!("unknown filter type {}", str)))
        }
    }
}

impl Default for ImageFilter {
    fn default() -> Self {
        ImageFilter{filter: FilterType::Nearest}
    }
}

pub struct OutputFormat {
    format: ImageOutputFormat,
}

impl OutputFormat {
    pub fn parse(str: &str) -> Result<OutputFormat, ImageError> {
        match str.to_lowercase().as_str() {
            "png" => Ok(OutputFormat{format: ImageOutputFormat::Png}),
            "jpeg" => Ok(OutputFormat{format: ImageOutputFormat::Jpeg(100)}),
            "gif" => Ok(OutputFormat{format: ImageOutputFormat::Gif}),
            _ => Err(ImageError::InvalidFormat(format!("can't convert to {}", str)))
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat{format: ImageOutputFormat::Png}
    }
}