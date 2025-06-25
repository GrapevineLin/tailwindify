//! CLI应用的错误类型定义
//!
//! 使用 thiserror 定义具体的错误类型，便于错误分类和处理

use thiserror::Error;

/// 应用程序的主要错误类型
#[derive(Error, Debug)]
pub enum TailwindifyError {
    #[error("无法读取目录: {path}")]
    DirectoryReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("文件读取失败: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("文件写入失败: {path}")]
    FileWriteError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("正则表达式编译失败: {pattern}")]
    RegexError {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("线程执行失败")]
    ThreadError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("临时文件操作失败: {operation}")]
    TempFileError {
        operation: String,
        #[source]
        source: std::io::Error,
    },
}

/// 简化的结果类型别名
pub type Result<T> = anyhow::Result<T>;

/// 将标准IO错误转换为我们的错误类型的辅助函数
impl TailwindifyError {
    pub fn directory_read_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::DirectoryReadError {
            path: path.into(),
            source,
        }
    }

    pub fn file_read_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileReadError {
            path: path.into(),
            source,
        }
    }

    pub fn file_write_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileWriteError {
            path: path.into(),
            source,
        }
    }

    pub fn regex_error(pattern: impl Into<String>, source: regex::Error) -> Self {
        Self::RegexError {
            pattern: pattern.into(),
            source,
        }
    }

    pub fn temp_file_error(operation: impl Into<String>, source: std::io::Error) -> Self {
        Self::TempFileError {
            operation: operation.into(),
            source,
        }
    }
}
