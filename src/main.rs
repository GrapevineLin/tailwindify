use argh::FromArgs;
use std::fs::{self, File};
use std::io::Write;
use std::rc::Rc;
use std::thread::{self, JoinHandle};

mod rules;

#[derive(FromArgs)]
/// cmd arguments
struct Aruguments {
    /// 扫描路径
    #[argh(positional)]
    directory: String,
    /// 需要手动确认的原子式前缀
    #[argh(option, short = 'p', default = "String::from(\"__需要手动处理>\")")]
    warn_prefix: String,
    /// 需要手动确认的原子式后缀
    #[argh(option, short = 's', default = "String::from(\"__需要手动处理>\")")]
    warn_suffix: String,
}

// TODO 好像有些println被吞了
fn main() {
    let _args: Aruguments = argh::from_env();
    let args = Rc::new(_args);
    let start = std::time::Instant::now();
    run(args);
    let duration = start.elapsed();
    println!("运行耗时:{:?}", duration);
}
static EXTENSION_VEC: [&str; 5] = ["vue", "tsx", "jsx", "js", "ts"];
// TODO 多少才算合理的？
static GROUP_COUNT: usize = 80;
fn run(args: Rc<Aruguments>) {
    let directory = &args.directory;
    let directory_vec = process_directory(directory).expect("读取目录错误");
    println!("{:?}", directory_vec);
    // let transform_reg_vec = rules::get_transform_reg_vec();
    // TODO 统计处理文件次数
    let mut count = 0;
    // 分组，每个组{GROUP_COUNT}个
    let mut file_vec: Vec<Vec<String>> = Vec::new();
    for file_str in directory_vec {
        if file_vec.is_empty() || file_vec.last().expect("数组为空").len() >= GROUP_COUNT {
            file_vec.push(Vec::new());
        };
        let group_vec = file_vec.last_mut().expect("数组为空");
        group_vec.push(String::clone(&file_str));
    }
    let mut handle_vec: Vec<JoinHandle<()>> = Vec::new();
    for group_vec in file_vec {
        let handle = thread::spawn(|| {
            for file_str in group_vec {
                if let Some(_) = replace_file_content(&file_str) {
                    // count = count + 1;
                }
            }
        });
        handle_vec.push(handle);
    }

    for handle in handle_vec {
        handle.join().expect("线程错误");
    }

    println!("一共处理了{}个文件", count)
}

/// 读取目录下的所有文件
fn process_directory(directory: &str) -> Result<Vec<String>, std::io::Error> {
    let result = fs::read_dir(directory);
    match result {
        Ok(files) => {
            let mut result_string_vec: Vec<String> = Vec::new();
            // 遍历目录
            for file_result in files {
                match file_result {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        let path_str = path.to_str();
                        // 如果是文件就加到集合里，如果是目录就继续遍历
                        if let Some(v) = path_str {
                            if path.is_file() {
                                if let Some(extension) = path.extension() {
                                    if let Some(extension_str) = extension.to_str() {
                                        if EXTENSION_VEC
                                            .iter()
                                            .any(|&x| x.eq_ignore_ascii_case(extension_str))
                                        {
                                            result_string_vec.push(String::from(v));
                                        } else {
                                            println!("格式不符合:{}", v)
                                        }
                                    }
                                }
                            } else if path.is_dir() {
                                if let Ok(mut sub_result) = process_directory(v) {
                                    result_string_vec.append(&mut sub_result);
                                }
                            }
                        }
                    }
                    Err(err) => return Result::Err(err),
                }
            }
            Ok(result_string_vec)
        }
        Err(err) => Result::Err(err),
    }
}

// 读取文件内容
fn replace_file_content(file_str: &str) -> Option<()> {
    match fs::read_to_string(&file_str) {
        Ok(content) => {
            let mut content = content;
            let mut has_replated = false;
            // TODO 共享一个配置 而不是各自创建
            let transform_reg_vec = rules::get_transform_reg_vec();

            for reg_item in &transform_reg_vec {
                let replaced = reg_item.reg.replace_all(&content, &reg_item.transform_fn);
                if replaced != content {
                    has_replated = true;
                    content = replaced.to_string();
                }
            }
            if has_replated {
                let mut file = match File::create(&file_str) {
                    Ok(file) => file,
                    Err(error) => {
                        println!("创建文件失败:{}", error);
                        return None;
                    }
                };
                match file.write_all(content.as_bytes()) {
                    Ok(_) => {
                        println!("写入文件{}成功", file_str);
                        return Some(());
                    }
                    Err(error) => {
                        println!("回写文件失败:{}", error);
                        return None;
                    }
                };
            } else {
                println!("文件没有匹配:{}", file_str);
                return None;
            }
        }
        Err(error) => {
            println!("读取文件失败:{}", error);
            return None;
        }
    }
}
