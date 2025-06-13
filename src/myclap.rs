use std::path::PathBuf;
use clap::Parser;
use tracing::Level;

use crate::error::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// 设置工作目录
    #[arg(short, long, value_name = "directory")]
    pub directory: Option<PathBuf>,

    /// 监听的端口号
    #[arg(short, long, value_name = "port",default_value_t = 3000)]
    pub port: u16,

    /// 启用调试信息
    #[arg(long,short, action = clap::ArgAction::Count)]
    pub loglevel: u8,

    /// 设置图片的宽度，该设置决定了瀑布流的宽度
    #[arg(long,short,default_value_t = 350)]
    pub width: usize,

}

pub struct Config{
    pub directory:PathBuf,
    pub port: u16,
    pub loglevel: Level,
    pub width: usize,
}

impl Config {
    pub fn from_parser(cli:Cli)->Result<Self,Error>{
        let port = cli.port;
    let workdir = match cli.directory {
        Some(d) => d,
        None => match std::env::current_dir() {
            Ok(d) => d,
            Err(_) => return Err(Error::CannotGetWorkDir),
        },
    };
    let loglevel = match cli.loglevel {
        1 => tracing::Level::ERROR,
        2 => tracing::Level::WARN,
        0|3 => tracing::Level::INFO,
        4 => tracing::Level::DEBUG,
        5.. => tracing::Level::TRACE,
    };
    Ok(Self { directory: workdir, port, loglevel: loglevel, width: cli.width })
    }
}