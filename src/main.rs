use argh::FromArgs;
use colored::*;
use log::{debug, error, info, warn};
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

mod error;
mod rules;

use error::{Result, TailwindifyError};

const DEFAULT_WARN_PREFIX: &str = "__需要手动处理>";
const DEFAULT_WARN_SUFFIX: &str = "<需要手动处理__";

#[derive(FromArgs, Clone)]
/// 启动参数
struct Arguments {
    /// 扫描路径
    #[argh(positional)]
    directory: String,
    /// 需要手动确认的原子式前缀
    #[argh(option, short = 'p', default = "DEFAULT_WARN_PREFIX.to_string()")]
    warn_prefix: String,
    /// 需要手动确认的原子式后缀
    #[argh(option, short = 's', default = "DEFAULT_WARN_SUFFIX.to_string()")]
    warn_suffix: String,
    /// 启用详细输出
    #[argh(switch, short = 'v')]
    verbose: bool,
    /// 启用调试模式
    #[argh(switch, short = 'd')]
    debug: bool,
}

fn main() {
    // 设置Windows控制台UTF-8编码
    #[cfg(windows)]
    unsafe {
        winapi::um::wincon::SetConsoleOutputCP(65001); // 65001 是 UTF-8 的代码页
    }

    let args: Arguments = argh::from_env();

    // 初始化日志系统
    init_logger(&args);

    let start = std::time::Instant::now();

    // 运行主程序逻辑，如果出错则优雅地处理错误
    if let Err(e) = run(args) {
        error!("{}", format!("程序执行失败: {}", e).red().bold());

        // 显示错误链
        let mut source = e.source();
        while let Some(err) = source {
            error!("{}", format!("  原因: {}", err).red());
            source = err.source();
        }

        std::process::exit(1);
    }

    let duration = start.elapsed();
    info!("{}", format!("✅ 运行耗时: {:?}", duration).green().bold());
}

/// 初始化日志系统
fn init_logger(args: &Arguments) {
    let log_level = if args.debug {
        "debug"
    } else if args.verbose {
        "info"
    } else {
        "warn"
    };

    unsafe {
        std::env::set_var("RUST_LOG", log_level);
    }
    env_logger::init();

    debug!("日志系统已初始化，级别: {}", log_level);
}

// TODO: 每次都要遍历数组进行比较;优化建议: 使用 HashSet
static EXTENSION_VEC: [&str; 5] = ["vue", "tsx", "jsx", "js", "ts"];

fn run(args: Arguments) -> Result<()> {
    info!("{}", "🚀 开始扫描文件...".blue().bold());

    let directory = &args.directory;
    let directory_vec = process_directory(directory)?;
    let total_count = directory_vec.len();

    if total_count == 0 {
        warn!("{}", "⚠️  没有找到任何符合条件的文件".yellow());
        return Ok(());
    }

    info!("{}", format!("📁 找到 {} 个文件", total_count).cyan());

    // 使用原子计数器跟踪处理进度
    let processed_count = Arc::new(AtomicUsize::new(0));
    let group_count = calculate_group_count(total_count);

    // 给线程分组
    let file_groups = create_file_groups(directory_vec, group_count);
    let mut handles: Vec<JoinHandle<Result<usize>>> = Vec::new();

    info!(
        "{}",
        format!("🔧 启动 {} 个工作线程", file_groups.len()).cyan()
    );

    for (group_id, group_vec) in file_groups.into_iter().enumerate() {
        let args_clone = args.clone();
        let counter = Arc::clone(&processed_count);

        let handle = thread::spawn(move || -> Result<usize> {
            debug!("线程 {} 开始处理 {} 个文件", group_id, group_vec.len());
            let mut local_count = 0;

            for file_str in group_vec {
                match replace_file_content(&args_clone, &file_str) {
                    Ok(()) => {
                        local_count += 1;
                        let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        info!(
                            "{}",
                            format!("✅ [{}/{}] {}", current, total_count, file_str).green()
                        );
                    }
                    Err(e) => {
                        warn!("{}", format!("⚠️  跳过文件 {}: {}", file_str, e).yellow());
                    }
                }
            }

            debug!("线程 {} 完成，处理了 {} 个文件", group_id, local_count);
            Ok(local_count)
        });

        handles.push(handle);
    }

    // 等待所有线程完成并收集结果
    let mut total_processed = 0;
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.join() {
            Ok(result) => match result {
                Ok(count) => {
                    total_processed += count;
                    debug!("线程 {} 成功完成", i);
                }
                Err(e) => {
                    error!("{}", format!("线程 {} 执行出错: {}", i, e).red());
                    return Err(e);
                }
            },
            Err(_) => {
                let err = TailwindifyError::ThreadError {
                    source: format!("线程 {} panic", i).into(),
                };
                error!("{}", format!("{}", err).red());
                return Err(err.into());
            }
        }
    }

    info!(
        "{}",
        format!(
            "🎉 扫描完成！共扫描 {} 个文件，成功处理 {} 个文件",
            total_count, total_processed
        )
        .green()
        .bold()
    );

    Ok(())
}

/// 读取目录下的所有文件
fn process_directory(directory: &str) -> Result<Vec<String>> {
    debug!("正在扫描目录: {}", directory);

    let entries = fs::read_dir(directory)
        .map_err(|e| TailwindifyError::directory_read_error(directory, e))?;

    let mut result_files = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| TailwindifyError::directory_read_error(directory, e))?;

        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if EXTENSION_VEC
                        .iter()
                        .any(|&x| x.eq_ignore_ascii_case(ext_str))
                    {
                        //  优先尝试零开销转换
                        if let Some(utf8_path) = path.to_str() {
                            // 合法 UTF-8 路径，直接克隆字符串
                            result_files.push(utf8_path.to_owned());
                        } else {
                            // 含非 UTF-8 字符，需要转义处理
                            result_files.push(path.to_string_lossy().into_owned());
                        }
                    } else {
                        debug!("跳过不支持的文件格式: {}", path.display());
                    }
                }
            }
        } else if path.is_dir() {
            // 递归处理子目录
            let path_str = path.to_string_lossy();
            let mut sub_files = process_directory(&path_str)?;
            result_files.append(&mut sub_files);
        }
    }

    Ok(result_files)
}

/// 创建文件分组
fn create_file_groups(files: Vec<String>, group_size: usize) -> Vec<Vec<String>> {
    let mut groups = Vec::new();
    let mut current_group = Vec::new();

    for file in files {
        if current_group.len() >= group_size {
            groups.push(current_group);
            current_group = Vec::new();
        }
        current_group.push(file);
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

/// 安全地替换文件内容
fn replace_file_content(args: &Arguments, file_path: &str) -> Result<()> {
    debug!("开始处理文件: {}", file_path);

    let content = fs::read_to_string(file_path)
        .map_err(|e| TailwindifyError::file_read_error(file_path, e))?;

    let mut current_content = content;
    let mut has_changes = false;

    // FIXME: 获取转换规则，应该只初始化一次，而不是每次都创建
    let transformation_rules = rules::get_transform_reg_vec(args)?;

    for rule in &transformation_rules {
        let replaced = rule.reg.replace_all(&current_content, &rule.transform_fn);
        // 只有当实际发生替换时才更新内容
        if let std::borrow::Cow::Owned(replaced_content) = replaced {
            has_changes = true;
            current_content = replaced_content;
        }
    }

    if has_changes {
        // 安全写入：先写临时文件，然后重命名
        write_file_safely(file_path, &current_content)?;
        debug!("文件更新成功: {}", file_path);
        Ok(())
    } else {
        debug!("文件无需更新: {}", file_path);
        Err(anyhow::anyhow!("文件内容无变化"))
    }
}

/// 安全地写入文件（使用临时文件）
fn write_file_safely(file_path: &str, content: &str) -> Result<()> {
    // 相比于format! 拼接 性能更好，可读性稍差
    let mut temp_path = String::with_capacity(file_path.len() + 4);
    temp_path.push_str(file_path);
    temp_path.push_str(".tmp");

    // 写入临时文件
    {
        let mut temp_file = File::create(&temp_path)
            .map_err(|e| TailwindifyError::temp_file_error("创建临时文件", e))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| TailwindifyError::temp_file_error("写入临时文件", e))?;

        temp_file
            .sync_all()
            .map_err(|e| TailwindifyError::temp_file_error("同步临时文件", e))?;
    }

    // 原子性地替换原文件
    fs::rename(&temp_path, file_path)
        .map_err(|e| TailwindifyError::file_write_error(file_path, e))?;

    Ok(())
}

fn calculate_group_count(total_files: usize) -> usize {
    let cpus = num_cpus::get();
    let group_count = std::cmp::max(1, total_files / cpus);
    debug!(
        "总文件数: {}, CPU核心数: {}, 每个线程处理文件数: {}",
        total_files, cpus, group_count
    );
    group_count
}
