#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("无法获取目录")]
    CannotGetItemList(#[from] std::io::Error),
    #[error("路径穿越")]
    PathTraversal,
    #[error("路径不合法")]
    PathNotValid,
    #[error("无法获取工作目录")]
    CannotGetWorkDir
}
