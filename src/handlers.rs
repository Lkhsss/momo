use askama::Template;
use axum::{
    body::Bytes,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use tokio::fs::{self};
use tracing::{info, warn};

use crate::{
    CONFIG,
    error::Error,
    filetype::{self, FileType, IMAGE_FILE_EXTENSION, ImageType, ToContentType},
    template::{ImageFallTemplate, Imgs},
};

pub async fn handler(uri: Option<axum::extract::Path<String>>) -> impl IntoResponse {
    let request_path = match uri {
        Some(u) => PathBuf::from(u.0),
        None => PathBuf::from("/"),
    };

    let base_path = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("无法获取工作目录。\n原因：{e}"),
            )
                .into_response();
        }
    };
    let uri = Uri::frompath(&request_path);

    let path = match uri.0.as_str() {
        //看看是否为请求网站更目录目录（即工作目录根目录）
        "/" => base_path,
        _ => format_path(base_path.join(&request_path)),
    };

    let request_uri = UriBox::new(path, uri);
    match request_uri.path.try_exists() {
        // 尝试判断是否存在，避免硬件错误
        Ok(status) => {
            //判断是否存在
            if status {
                //存在
                if request_uri.path.is_file() {
                    file_handler(request_uri).await.into_response()
                } else if request_uri.path.is_dir() {
                    dir_handler(request_uri).await.into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "不是文件也不是目录").into_response()
                }
            } else {
                //不存在
                (StatusCode::NOT_FOUND, "目标不存在").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)).into_response(),
    }
}

pub async fn dir_handler(dir: UriBox) -> impl IntoResponse {
    //获取请求目录的列表
    let items = match get_item(dir) {
        Ok(o) => o,
        Err(e) => match e {
            Error::CannotGetItemList(error) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
            }
            Error::PathTraversal => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            Error::PathNotValid => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            Error::CannotGetWorkDir => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        },
    };

    let mut imgs = Vec::new();
    for i in items {
        if let Some(ext) = i.path.extension() {
            let ext = match ext.to_str() {
                Some(e) => e,
                None => panic!(),
            };
            if filetype::ImageType::is_image_file(ext) {
                // info!("{}",i.uri.0);
                imgs.push(Imgs::new(i.uri.0));
            }
        }
    }

    if imgs.len() == 0 {
        return (
            StatusCode::OK,
            Html(include_bytes!("../templates/NoResources.html")),
        )
            .into_response();
    }

    // 返回模板数据
    let html = match (ImageFallTemplate {
        imgs,
        column_width: CONFIG.width,
    })
    .render()
    {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, ("无法返回模板文件")).into_response(),
    };
    (StatusCode::OK, Html(html)).into_response()
}

/// # 读取文件并且设置CONTENT_TYPE
/// ### 需要确定文件存在！
pub async fn file_handler(file: UriBox) -> impl IntoResponse {
    let extension = file.path.extension();
    let header = match extension {
        //如果有后缀
        Some(e) => {
            let extension = e.to_string_lossy().to_string().to_lowercase(); //全部转小写,避免出错

            let content_type = extension_handler(extension).get_content_type().to_string();
            [(header::CONTENT_TYPE, content_type)]
        } //如果没后缀
        None => [(
            header::CONTENT_TYPE,
            FileType::Other.get_content_type().to_string(),
        )],
    };
    // 异步读取文件
    match fs::read(&file.path).await {
        Ok(data) => (header, Bytes::from(data)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("文件读取出错, 原因：{e}"),
        )
            .into_response(),
    }
}

fn extension_handler(ext: String) -> FileType {
    if IMAGE_FILE_EXTENSION.contains(&ext.to_lowercase().as_str())
    //如果为图片格式
    {
        FileType::Image(image_extension_handler(ext))
    } else {
        FileType::Other
    }
}
// 全部自动转小写
fn image_extension_handler(extension: impl ToString) -> ImageType {
    match &extension.to_string().to_lowercase()[..] {
        "apng" => ImageType::Apng,
        "jpeg" | "jpg" | "jfif" | "pjpeg" | "pjp" => ImageType::Jpeg,
        "png" => ImageType::Png,
        "svg" => ImageType::Svg,
        "webp" => ImageType::WebP,
        "bmp" => ImageType::Bmp,
        "ico" | "cur" => ImageType::Ico,
        "tif" | "tiff" => ImageType::Tiff,
        _ => panic!(),
    }
}

fn format_path(ori: PathBuf) -> PathBuf {
    let mut new_path = PathBuf::new();
    for p in ori.components() {
        new_path.push(p);
    }
    new_path
}

fn get_item(p: UriBox) -> Result<Vec<UriBox>, Error> {
    let mut items = Vec::new();
    let base_path = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => {
            return Err(Error::CannotGetWorkDir);
        }
    };
    for entry in std::fs::read_dir(&p.path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            match reslove_relative_path(&base_path, entry.path()) {
                Ok(uri) => items.push(UriBox {
                    path: entry.path(),
                    uri,
                }),
                Err(e) => return Err(e),
            }; //计算相对路径
        }
    }
    Ok(items)
}

#[derive(Debug, Clone)]
pub struct Uri(String);

impl Uri {
    //直接转化为Uri
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    fn frompath(url: impl AsRef<Path>) -> Self {
        if url.as_ref() == PathBuf::from("/") {
            Self("/".into())
        } else {
            let mut new_url = String::new();
            for i in url.as_ref().components() {
                new_url.push('/');
                new_url.push_str(&format!("{}", i.as_os_str().to_string_lossy()));
            }
            Self(new_url)
        }
    }

    //将迭代器转为Uri
    fn from_iter<T: Into<String>>(slice: impl Iterator<Item = T>) -> Self {
        let mut uri = String::new();
        for i in slice {
            uri.push('/');
            uri.push_str(&i.into());
        }
        Self::new(uri)
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//处理url和本地目录的关系
#[derive(Debug, Clone)]
pub struct UriBox {
    path: PathBuf,
    uri: Uri,
}

impl UriBox {
    fn new(path: PathBuf, uri: Uri) -> Self {
        Self { path, uri }
    }
}

/// 计算相对路径
fn reslove_relative_path(
    base_path: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
) -> Result<Uri, Error> {
    let mut base_path_components: Vec<String> = match base_path.as_ref().canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(Error::PathNotValid),
    }
    .components()
    .map(|x| x.as_os_str().to_string_lossy().to_string())
    .collect();

    let relative_path_canonicalize = match relative_path.as_ref().canonicalize() {
        Ok(p) => p,
        Err(_) => return Err(Error::PathNotValid),
    };
    let mut relative_path_components: Vec<String> = relative_path_canonicalize
        .components()
        .map(|x| x.as_os_str().to_string_lossy().to_string())
        .collect();
    base_path_components.retain(|x| x != "\\");
    relative_path_components.retain(|x| x != "\\");

    if relative_path_components.len() < base_path_components.len() {
        //如果请求路径短于基础路径那肯定是非法的
        warn!("路径穿越攻击！");
        return Err(Error::PathTraversal);
    }

    for i in 0..(base_path_components.len()) {
        if base_path_components[i] != relative_path_components[i] {
            return Err(Error::PathTraversal);
        }
    }
    let result: Vec<String> = relative_path_components
        .drain((base_path_components.len())..)
        .collect();
    Ok(Uri::from_iter(result.iter()))
}
