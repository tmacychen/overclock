use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

pub fn handle_init(name: &str, path: Option<&str>) -> Result<()> {
    let base_path = path.unwrap_or(".");
    let target_path = Path::new(base_path).join(name);

    if target_path.exists() {
        return Err(anyhow::anyhow!(
            "目录 '{}' 已存在，请使用不同的项目名称或路径",
            target_path.display()
        ));
    }

    let template_path = get_template_path()?;

    if !template_path.exists() {
        return Err(anyhow::anyhow!(
            "模板目录不存在: {}",
            template_path.display()
        ));
    }

    println!("{} 项目 '{}'...", "初始化".green(), name.cyan());

    std::fs::create_dir_all(&target_path)
        .with_context(|| format!("无法创建目录: {}", target_path.display()))?;

    let mut options = fs_extra::dir::CopyOptions::new();
    options.content_only = true;
    options.overwrite = false;

    fs_extra::dir::copy(&template_path, &target_path, &options)
        .with_context(|| format!("无法复制模板到: {}", target_path.display()))?;

    println!(
        "{} 项目已创建: {}",
        "✓".green(),
        target_path.display().to_string().cyan()
    );
    println!();
    println!("下一步:");
    println!("  cd {}", name);
    println!("  查看 .ai/ 目录了解项目结构");

    Ok(())
}

fn get_template_path() -> Result<std::path::PathBuf> {
    let exe_path = std::env::current_exe().context("无法获取可执行文件路径")?;

    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("无法获取可执行文件目录"))?;

    let template_path = exe_dir.join("template");
    if template_path.exists() {
        return Ok(template_path);
    }

    let cwd_template = std::env::current_dir()
        .context("无法获取当前工作目录")?
        .join("template");

    if cwd_template.exists() {
        return Ok(cwd_template);
    }

    Ok(cwd_template)
}
