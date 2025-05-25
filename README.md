# Tailwindify

![Rust](https://img.shields.io/badge/Rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)
![UnoCSS](https://img.shields.io/badge/UnoCSS-%23000000.svg?style=for-the-badge&logo=unocss&logoColor=white)

## 背景

在开发过程中，可能会使用 [UnoCSS](https://github.com/unocss/unocss) 来定义自定义的原子类（如 `pl8`、`font-weight-600`、`fs-16`）。然而，当项目需要迁移到 [Tailwind CSS](https://tailwindcss.com/) 时，这些自定义的原子类可能与 Tailwind 的规范不匹配，导致样式不一致或需要手动调整。

`Tailwindify` 是一个用 Rust 编写的工具，旨在自动化地将 某些非 类Tailwind 风格的原子类转换为 Tailwind CSS 规范的原子类，从而简化迁移过程并减少手动工作量。

## 功能

- **原子类转换**：将 UnoCSS 风格的原子类（如 `pl8`、`font-weight-600`）转换为 Tailwind 规范的形式（如 `pl-8`、`font-600`）。
- **批量处理**：支持对代码文件或项目目录进行批量转换。
- **自定义规则**：允许用户定义额外的转换规则，以适应项目特定的需求。

## 安装

```bash
# 通过 Cargo 安装
cargo install tailwindify
```