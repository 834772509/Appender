use crate::core::{addResource, exportResource, ResourceHead, CompressMode, findResourcesConfig};
use std::path::{PathBuf};
use std::{fs};

lazy_static! {
    static ref ID: String = "1".to_string();
    static ref BASE_PATH: PathBuf = PathBuf::from(r"D:\Project\back-end\Appender\test");
    static ref TARGET_FILE: PathBuf = BASE_PATH.join("Notepad.exe");
    static ref SOURCE_FILE: PathBuf = BASE_PATH.join("hh.exe");
    static ref OUTPUT_PATH: PathBuf = BASE_PATH.join("输出.exe");
}

/// 增加资源测试
#[test]
fn addResourceTest() {
    addResource((&*TARGET_FILE).as_ref(), (&*SOURCE_FILE).as_ref(), &ID, None, None).unwrap();
    println!("资源增加成功");
}

/// 释放资源测试
#[test]
fn exportResourceTest() {
    exportResource((&*TARGET_FILE).as_ref(), &ID, OUTPUT_PATH.as_ref()).unwrap();
    println!("资源释放成功");
}

/// 自动测试
#[test]
fn autoTest() {
    let testPath = TARGET_FILE.parent().unwrap().join("test");

    fs::remove_dir_all(&testPath);
    fs::create_dir(&testPath);

    let testTargetFile = testPath.join(TARGET_FILE.file_name().unwrap());
    let testOutputFile = testPath.join(OUTPUT_PATH.file_name().unwrap());
    fs::copy(TARGET_FILE.clone(), &testTargetFile);

    addResource((&*testTargetFile).as_ref(), (&*SOURCE_FILE).as_ref(), &ID, None, None).unwrap();
    exportResource((&*testTargetFile).as_ref(), &ID, &*testPath).unwrap();
    println!("测试成功");
}

extern crate test;
use test::Bencher;
/// 性能测试 - 释放资源
#[bench]
fn bench_Test(b: &mut Bencher) {
    b.iter(|| exportResourceTest());
}

/// 实例测试
#[test]
fn Test() {
    // let defaultResourceHead = ResourceHead::default();

    // let resource = ResourceHead::new("1", 123, "aaa.exe",CompressMode::None);
    let resource = ResourceHead::new("1", 39278156, 100,"day01_21.zip", CompressMode::None);
    // println!("{:?}", resource);

    // let bin = &resource.to_bytes().unwrap();
    // println!("{:?}", bin);

    // Length
    // println!("{:?}", bin.len());
    // println!("{}", defaultResourceHead.getLen());

    // println!("{:?}", format!("{:^length$}", "abcd", length = 255).chars().count());
    // println!("{:?}", format!("{:^length$}", "我我我我w", length = 255).chars().count());
    println!("{:?}", format!("{:^length$}", "abcd", length = 5).as_bytes());
    let str = "abcd";
    println!("{:?}", format!("{:^length$}", str, length = 5 - str.len() + str.chars().count()).as_bytes());
}

use serde::{Serialize, Deserialize};
use crate::util::{decompressFile, compressionFile};

#[test]
fn test2() {
    // println!("{:?}", getResourceList(&*PathBuf::from(r"C:\Users\Administrator.W10-20201229857\Desktop\新建文件夹\test\Notepad.exe")));

    let aa = Astruct {
        Name: "".to_string(),
        CompressMode: CompressMode::Compress,
    };
    println!("{:?}", bincode::serialize(&aa));
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Astruct {
    Name: String,
    CompressMode: CompressMode,
}

// 压缩、解压 测试
#[test]
fn CompressionTest() {
    let sourcePath = PathBuf::from(r"D:\Project\back-end\Appender\test\hh.exe");
    let targetPath = PathBuf::from(r"D:\Project\back-end\Appender\test\temp");
    let dePath = PathBuf::from(r"D:\Project\back-end\Appender\test\temp.exe");

    println!("=============压缩=============");
    compressionFile(&*sourcePath, &*targetPath, 5);
    println!("=============解压=============");
    decompressFile(&*targetPath, &*dePath);
}

#[test]
fn test4(){
    findResourcesConfig(&*PathBuf::from(r"D:\Project\FirPE\Win10PE\test.wim"), |startSize: usize,config: &ResourceHead| {
        println!("{:?}", config);
    });
    println!("============");
}
