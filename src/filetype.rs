use std::fmt::Display;

pub const IMAGE_FILE_EXTENSION: [&str; 16] = [
    "apng", "avif", "gif", "jpg", "jpeg", "jfif", "pjpeg", "pjp", "png", "svg", "webp", "bmp",
    "ico", "cur", "tif", "tiff",
];

#[derive(Debug, PartialEq, Eq)]
pub enum FileType {
    Image(ImageType),
    Other,
    //TODO 完成其他类型
}

impl ToContentType for FileType {
    fn get_content_type(&self) -> impl std::fmt::Display {
        match self {
            FileType::Image(image) => image.get_content_type().to_string(),
            FileType::Other => "application/octet-stream".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum ImageType {
    Apng,
    Avif,
    Gif,
    Jpeg,
    Png,
    Svg,
    WebP,
    Bmp,
    Ico,
    Tiff,
}



impl Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ImageType::Apng => "apng",
            ImageType::Avif => "avif",
            ImageType::Gif => "gif",
            ImageType::Jpeg => "jpeg",
            ImageType::Png => "png",
            ImageType::Svg => "svg",
            ImageType::WebP => "webp",
            ImageType::Bmp => "bmp",
            ImageType::Ico => "ico",
            ImageType::Tiff => "tiff",
        };
        write!(f, "{}", s)
    }
}

impl ImageType {
    //全转小写，方便比对
    pub fn is_image_file(extension: &str) -> bool {
        IMAGE_FILE_EXTENSION.contains(&extension.to_lowercase().as_str())
    }
}

impl ToContentType for ImageType {
    fn get_content_type(&self) -> impl Display {
        match self {
            ImageType::Apng => "image/apng",
            ImageType::Avif => "image/avif",
            ImageType::Gif => "image/gif",
            ImageType::Jpeg => "image/jpeg",
            ImageType::Png => "image/png",
            ImageType::Svg => "image/svg+xml",
            ImageType::WebP => "image/webp",
            ImageType::Bmp => "image/bmp",
            ImageType::Ico => "image/x-icon",
            ImageType::Tiff => "image/tiff",
        }
    }
}

/// 实现此trait后可以通过结构体获取Content-Type
pub trait ToContentType {
    fn get_content_type(&self) -> impl Display;
}
