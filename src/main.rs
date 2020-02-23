use std::fs::{self, DirEntry, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Command;

use fs_extra::dir::{self, copy};
use fs_extra::{copy_items, remove_items};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::{self, RegKey};

fn main() {
    #[cfg(windows)]
    {
        println!("Reading some system info...");
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cur_ver = hklm.open_subkey(r"HKCU\Environment").unwrap();
//        let pf: String = cur_ver.get_value("ProgramFilesDir").unwrap();
//        let dp: String = cur_ver.get_value("DevicePath").unwrap();
//        println!("ProgramFiles = {}\nDevicePath = {}", pf, dp);
        let info = cur_ver.query_info().unwrap();
        println!("info = {:?}", info);
    }

    // 获取当前目录
    let current_exe = &std::env::current_exe().unwrap();
    let current_exe = Path::new(current_exe);

    let path = current_exe.parent().unwrap();
    let current_dir = path.to_str().unwrap();

    // 获取home目录
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap().trim_end_matches("/");

    // 判断是否已有安装目录
    let panda_dir_string = fix_filepath(format!("{}/.panda_api/", home_dir));
    let panda_dir = Path::new(&panda_dir_string);
    if panda_dir.exists() {
        // 如果文件夹存在，删除重装
        let mut from_paths = vec![&panda_dir_string];
        let _r = remove_items(&from_paths);
        //        println!("delete r {:?}", r);
    }

    // 如果文件夹不存在，创建文件夹
    match std::fs::create_dir_all(&panda_dir_string) {
        Ok(_) => (),
        Err(e) => {
            println!("create folder failed {} {:?}", &panda_dir_string, e);
        }
    }

    let options = dir::CopyOptions::new(); //Initialize default values for CopyOptions
    let install_files = if cfg!(target_os = "windows") {
        ["panda.exe", "theme"]
    } else {
        ["panda", "theme"]
    };
    let mut from_paths: Vec<String> = Vec::new();
    for file in &install_files {
        from_paths.push(format!("{}/Contents/{}", current_dir, file));
    }
    let r = copy_items(&from_paths, &panda_dir_string, &options);

    if cfg!(target_os = "windows") {
        // 增加windows环境变量
    } else {
        // 获取使用的是哪种shell
        let output = Command::new("sh")
            .arg("-c")
            .arg("echo $SHELL")
            .output()
            .expect("failed to execute process");

        let shell_name = String::from_utf8(output.stdout).unwrap();
        let shell_name = shell_name
            .trim()
            .trim_start_matches("/")
            .trim_start_matches("/");
        let mut profile_name = "".to_string();
        let shell_name_info: Vec<&str> = shell_name.split("/").collect();
        if let Some(shell_name) = shell_name_info.last() {
            profile_name = format!(".{}rc", shell_name);
        }

        let profile_filepath_string = format!("{}/{}", home_dir, profile_name);
        let profile_filepath = Path::new(&profile_filepath_string);

        let profile_content = r#"export PATH="$HOME/.panda_api:$PATH""#;
        // 编辑profile文件
        let mut has_profile_content = false;

        if profile_filepath.exists() {
            // 读取里面的内容，看是否已经有，没有的话就要编辑加上内容
            let mut content = fs::read_to_string(&profile_filepath_string)
                .expect(&format!("failed to read file {}", &profile_filepath_string));
            if content.contains(profile_content) {
                has_profile_content = true;
            }
        }

        if !has_profile_content {
            let mut file_options = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open(&profile_filepath_string);

            match file_options {
                Ok(mut file) => {
                    let new_content = format!("{}\n", profile_content);
                    file.write_all(new_content.as_bytes()).expect(&format!(
                        "failed to write data to file {}",
                        &profile_filepath_string
                    ));
                }
                Err(e) => {
                    panic!("{:?}", e);
                }
            }
        }

        let profile_list = [".zshrc", ".bashrc", ".cshrc"];

        for profile_file in &profile_list {
            let profile_filepath_string = format!("{}/{}", home_dir, profile_file);
            let profile_filepath = Path::new(&profile_filepath_string);

            if profile_filepath.exists() {
                // 读取里面的内容，看是否已经有，没有的话就要编辑加上内容
                let mut content = fs::read_to_string(&profile_filepath_string)
                    .expect(&format!("failed to read file {}", &profile_filepath_string));
                if content.contains(profile_content) {
                    continue;
                } else {
                    let mut file_options = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .append(true)
                        .open(&profile_filepath_string);

                    match file_options {
                        Ok(mut file) => {
                            let new_content = format!("{}\n", profile_content);
                            file.write_all(new_content.as_bytes()).expect(&format!(
                                "failed to write data to file {}",
                                &profile_filepath_string
                            ));
                        }
                        Err(e) => {
                            println!("不存在 {} {:?}", profile_filepath_string, e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    println!("Congratulations!\nPanda api install done!\nYou can run pana command in your api docs folder now.");
}

fn fix_filepath(filepath: String) -> String {
    filepath
        .replace("(", r"\(")
        .replace(")", r"\)")
        .replace(" ", r"\ ")
}
