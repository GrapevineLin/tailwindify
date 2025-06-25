use super::Arguments;
use crate::error::{Result, TailwindifyError};
use regex::{Captures, Regex};

pub struct TransformReg {
    pub reg: Regex,
    pub transform_fn: Box<dyn Fn(&Captures) -> String + Send + Sync>,
}

pub fn get_transform_reg_vec(arg: &Arguments) -> Result<Vec<TransformReg>> {
    let rules = vec![
        TransformReg {
            reg: Regex::new(r"(m|p)(t|r|b|l)(\d+)(r?)")
                .map_err(|e| TailwindifyError::regex_error(r"(m|p)(t|r|b|l)(\d+)(r?)", e))?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let property_type = caps.get(1).unwrap().as_str(); // m(margin) 或 p(padding)
                    let direction = caps.get(2).unwrap().as_str(); // t(top), r(right), b(bottom), l(left)
                    let size = caps.get(3).unwrap().as_str(); // 数字大小
                    let responsive = caps.get(4).map_or("", |m| m.as_str()); // 响应式标记 r
                    let result = format!("{}{}-{}{}", property_type, direction, size, responsive);
                    if responsive != "" {
                        log::info!("检测到r后缀！");
                        format!("{}{}{}", need_handle_prefix, result, need_handle_suffix)
                    } else {
                        result
                    }
                })
            },
        },
        TransformReg {
            reg: Regex::new(r"fs-?(\d+)(r)?")
                .map_err(|e| TailwindifyError::regex_error(r"fs-?(\d+)(r)?", e))?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let size = caps.get(1).unwrap().as_str();
                    let unit = caps.get(2).map_or("", |m| m.as_str());
                    let result = format!("text-{}{}", size, unit);
                    if unit == "r" {
                        log::info!("检测到r后缀！");
                        format!("{}{}{}", need_handle_prefix, result, need_handle_suffix)
                    } else {
                        result
                    }
                })
            },
        },
        TransformReg {
            reg: Regex::new(r"font-weight-(\d+)")
                .map_err(|e| TailwindifyError::regex_error(r"font-weight-(\d+)", e))?,
            transform_fn: Box::new(|caps: &Captures| {
                let size = caps.get(1).unwrap().as_str();
                format!("font-{}", size)
            }),
        },
        TransformReg {
            reg: Regex::new(r"(lh|line-height)-?(\d+)(p|r)?")
                .map_err(|e| TailwindifyError::regex_error(r"(lh|line-height)-?(\d+)(p|r)?", e))?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let size = caps.get(2).unwrap().as_str();
                    let unit = caps.get(3).map_or("", |m| m.as_str());
                    if unit == "r" {
                        log::info!("检测到r后缀！");
                        return format!(
                            "{}leading-{}r{}",
                            need_handle_prefix, size, need_handle_suffix
                        );
                    }
                    let unit_str = if unit == "p" { "%" } else { "" };
                    format!("leading-{}{}", size, unit_str)
                })
            },
        },
        TransformReg {
            reg: Regex::new(r"(border-radius-|br)-?(\d+)(p|r)?").map_err(|e| {
                TailwindifyError::regex_error(r"(border-radius-|br)-?(\d+)(p|r)?", e)
            })?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let size = caps.get(2).unwrap().as_str();
                    let unit = caps.get(3).map_or("", |m| m.as_str());
                    if unit == "r" {
                        log::info!("检测到r后缀！");
                        return format!(
                            "{}rounded-{}r{}",
                            need_handle_prefix, size, need_handle_suffix
                        );
                    }
                    let unit_str = if unit == "p" { "%" } else { "" };
                    format!("rounded-{}{}", size, unit_str)
                })
            },
        },
        // TODO 应该根据配置开关这个设置
        TransformReg {
            reg: Regex::new(r"opacity-(\d+)")
                .map_err(|e| TailwindifyError::regex_error(r"opacity-(\d+)", e))?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let size = caps.get(1).unwrap().as_str();
                    format!(
                        "{}opacity-{}{}",
                        need_handle_prefix, size, need_handle_suffix
                    )
                })
            },
        },
        // FIXME 当前转换规则过于简单，会错误转换很多不是颜色的原子式
        TransformReg {
            reg: Regex::new(r"c-?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})").map_err(|e| {
                TailwindifyError::regex_error(r"c-?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})", e)
            })?,
            transform_fn: {
                Box::new(|caps: &Captures| {
                    let color = caps.get(1).unwrap().as_str();
                    format!("text-#{}", color)
                })
            },
        },
        // tailwind 没有这个规则
        TransformReg {
            reg: Regex::new(r"grid-template-columns-(\d+)")
                .map_err(|e| TailwindifyError::regex_error(r"grid-template-columns-(\d+)", e))?,
            transform_fn: {
                let need_handle_prefix = arg.warn_prefix.clone();
                let need_handle_suffix = arg.warn_suffix.clone();
                Box::new(move |caps: &Captures| {
                    let size = caps.get(1).unwrap().as_str();
                    format!(
                        "{}grid-template-columns-{}{}",
                        need_handle_prefix, size, need_handle_suffix
                    )
                })
            },
        },
        // ellipsis
    ];

    Ok(rules)
}
