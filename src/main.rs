use std::fs::{self, DirEntry, File, OpenOptions};
use std::io::{self, BufReader, Read, Write, Error};
use std::path::Path;
use std::process::Command;

use fs_extra::dir::{self, copy};
use fs_extra::{copy_items, remove_items};

#[cfg(windows)]
use winapi;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::{self, RegKey};

fn main() {
    // 获取当前目录
    let current_exe = &std::env::current_exe().unwrap();
    let current_exe = Path::new(current_exe);

    let path = current_exe.parent().unwrap();
    let current_dir = path.to_str().unwrap();

    let split_s = if cfg!(target_os = "windows") {
        r"\"
    } else {
        "/"
    };

    // 获取home目录
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap().trim_end_matches(split_s);
    println!("home_dir {}", home_dir);
    // 判断是否已有安装目录
    let panda_dir_string = fix_filepath(format!("{1}{0}.panda_api{0}", split_s, home_dir));
    let panda_dir = Path::new(&panda_dir_string);
    if panda_dir.exists() {
        // 如果文件夹存在，删除重装
        let mut from_paths = vec![&panda_dir_string];
        let _r = remove_items(&from_paths);
        //        println!("delete r {:?}", r);
    }

    println!("panda_dir_string {}", panda_dir_string);

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
        from_paths.push(format!("{1}{0}Contents{0}{2}", split_s, current_dir, file));
    }
    match copy_items(&from_paths, &fix_filepath(panda_dir_string), &options) {
        Ok(r) => {
            println!("Copy files done.");
        }
        Err(e) => {
            println!("Copy files failed, install failed");
            return;
        }
    }

    let success_msg = "Congratulations!\nPanda api install done!\nYou can run pana command in your api docs folder now.";

    if cfg!(target_os = "windows") {
        // 增加windows环境变量
        #[cfg(windows)]
        {
            let hklm = RegKey::predef(HKEY_CURRENT_USER);
            let cur_ver = hklm.open_subkey("Environment").unwrap_or_else(|e| match e.kind() {
                io::ErrorKind::NotFound => panic!("Key doesn't exist"),
                io::ErrorKind::PermissionDenied => panic!("Access denied"),
                _ => panic!("{:?}", e),
            });

            let (reg_key, disp) = hklm.create_subkey("Environment").unwrap();

            let user_envs: String = if let Ok(p) = cur_ver.get_value("Path") {
                p
            } else {
                "".to_string()
            };

            let mut user_envs = user_envs.trim().trim_end_matches(";");
            let panda_dir_string = panda_dir_string.trim_end_matches(split_s);
            if user_envs.contains(panda_dir_string) {
//                println!("已经存在这个环境变量了");
            } else {
//                println!("还没有存在这个环境变量");
                let s = format!("{};{};", user_envs, panda_dir_string);
                match reg_key.set_value("Path", &s) {
                    Ok(r) => {
                        println!("reg ok");
                    }
                    Err(e) => {
                        println!("reg failed");
                    }
                }
            }
        }
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
    println!("{}", success_msg);

}

fn fix_filepath(filepath: String) -> String {
    filepath
        .replace("(", r"\(")
        .replace(")", r"\)")
        .replace(" ", r"\ ")
}
