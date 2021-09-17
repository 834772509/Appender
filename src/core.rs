use std::path::Path;
use std::error::Error;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, SeekFrom, Seek};
use std::convert::TryFrom;
use std::fs;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use memchr::memmem;
use crate::util::{compressionFile, decompressFile};


/// 缓冲区大小（512KB）
pub const BUFFER_SIZE: usize = 1024 * 512;

/// 资源最大大小（1024GB）
pub const MAX_LENGTH_SIZE: u64 = 1024 * 1024 * 1024 * 1024;

/// 最大id长度
pub const MAX_ID_LENGTH: usize = 64;

/// 最大文件名长度
pub const MAX_NAME_LENGTH: usize = 255;

/// 压缩模式
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum CompressMode {
    /// 无压缩
    None,
    /// 有压缩
    Compress,
}

/// 资源文件头
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceHead {
    /// 文件头魔数
    Head: Vec<u8>,
    /// 资源版本(不应与程序版本号绑定)
    Version: String,
    /// 资源ID
    Id: String,
    /// 资源文件名
    Name: String,
    /// 资源长度
    Length: String,
    /// 资源大小
    Size: String,
    /// 压缩模式
    Compress: CompressMode,
}

impl ResourceHead {
    pub(crate) fn default() -> Self {
        ResourceHead::new("", 0, 0, "", CompressMode::None)
    }

    pub fn new(id: &str, length: u64, size: u64, name: &str, CompressMode: CompressMode) -> Self {
        // 注意: 如有中文填充需要使用 指定字符串长度 - 字符串.len() + 字符串.chars().count()
        ResourceHead {
            Head: [
                // 非 ASCII 字符以防止来自文本文件的干扰
                vec![0x89].as_slice(),
                b"OverlayData".to_vec().as_slice(),
                vec![0x0d, 0x0a, 0x1a, 0x0a].as_slice()
            ].concat(),
            Id: format!("{:^length$}", id, length = MAX_ID_LENGTH - id.len() + id.chars().count()),
            Name: format!("{:^length$}", name, length = MAX_NAME_LENGTH - name.len() + name.chars().count()),
            Length: format!("{:0>length$}", length, length = MAX_LENGTH_SIZE.to_string().len()),
            Size: format!("{:0>length$}", size, length = MAX_LENGTH_SIZE.to_string().len()),
            Version: "1.0.0".to_string(),
            Compress: CompressMode,
        }
    }

    /// 获取文件头长度
    pub fn getLen(&self) -> usize {
        self.to_bytes().unwrap().len()
    }

    /// 获取文件头魔数（标识）
    pub fn getHead(&self) -> &Vec<u8> {
        &self.Head
    }

    /// 转换为字节
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(&self)
    }

    /// 将字节解析为当前数据
    pub fn from(data: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(data)
    }
}

/// 资源文件尾(ODEND)
const END_IDENTIFIER: [u8; 5] = [0x4F, 0x44, 0x45, 0x4E, 0x44];

/// 增加资源(Overlay 附加数据)
/// # 参数
/// 1. 目标文件
/// 2. 资源文件
/// 3. 资源ID（不可重复）
/// 4. 输出文件(可选)
pub fn addResource(targetFilePath: &Path, sourceFilePath: &Path, id: &str, compressionGrade: Option<u32>, outputPath: Option<&Path>) -> Result<(), Box<dyn Error>> {
    // 打开资源文件
    let sourceFilePath = if sourceFilePath.is_relative() { targetFilePath.parent().unwrap().join(sourceFilePath) } else { sourceFilePath.to_path_buf() };
    let mut sourceFile = File::open(&sourceFilePath)?;
    let sourceName = &sourceFilePath.file_name().unwrap().to_str().unwrap();

    // 处理压缩资源
    let tempFilePath = &*sourceFilePath.parent().unwrap().join("temp");
    if let Some(grage) = compressionGrade {
        compressionFile(&*sourceFilePath, tempFilePath, grage)?;
        sourceFile = File::open(&tempFilePath)?;
    }
    let sourceLength = sourceFile.metadata()?.len();

    //以追加模式打开目标文件
    let targetFilePath = if let Some(outputPath) = outputPath {
        // 处理相对路径
        let outputPath = if outputPath.is_relative() { targetFilePath.parent().unwrap().join(outputPath) } else { outputPath.to_path_buf() };
        fs::copy(targetFilePath, &outputPath)?;
        outputPath
    } else { targetFilePath.to_path_buf() };

    let mut targetFile = OpenOptions::new().append(true).open(targetFilePath)?;

    let compressMode = match compressionGrade.is_some() {
        true => CompressMode::Compress,
        false => CompressMode::None,
    };

    // 插入标识头
    let head = ResourceHead::new(id, sourceLength, sourceLength, sourceName, compressMode).to_bytes()?;
    if head.len() != ResourceHead::default().to_bytes()?.len() {
        return Err(Box::try_from("The resource information is not standard, please make sure that there are no Chinese symbols in the information").unwrap());
    }
    targetFile.write_all(&*head)?;

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];

    // 循环读取并写入资源文件
    loop {
        let nbytes = sourceFile.read(&mut buffer)?;
        targetFile.write_all(&buffer[..nbytes])?;
        if nbytes < buffer.len() { break; }
    }

    // 插入尾部标识
    targetFile.write_all(&END_IDENTIFIER)?;

    // 清除临时压缩资源
    if tempFilePath.exists(){
        fs::remove_file(&tempFilePath)?;
    }
    Ok(())
}

/// 释放资源
/// # 参数
/// 1. 目标文件
/// 2. 资源ID
/// 3. 输出路径
pub fn exportResource(targetFilePath: &Path, id: &str, outputPath: &Path) -> Result<(), Box<dyn Error>> {
    let defaultResourceHead = ResourceHead::default();

    // 打开目标文件
    let mut sourceFile = File::open(targetFilePath)?;

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut count = 0;

    loop {
        count += 1;
        let nbytes = &sourceFile.read(&mut buffer).unwrap();

        // 如果找到了资源
        // if let Some(size) = find_subsequence(&buffer, &*defaultResourceHead.getHead()) {
        if let Some(size) = memmem::Finder::new(&*defaultResourceHead.getHead()).find(&buffer) {
            // 资源文件头起始位置(我也不知道为什么要减8)
            let startSize = BUFFER_SIZE * (count - 1) + size - 8;

            let oldSize = sourceFile.stream_position().unwrap();

            // 偏移读取配置
            let mut configBuffer: Vec<u8> = vec![0; defaultResourceHead.getLen()];
            sourceFile.seek(SeekFrom::Start(startSize as u64))?;
            sourceFile.read_exact(&mut configBuffer)?;

            let config = ResourceHead::from(&configBuffer);
            if let Err(_e) = config {
                continue;
            }
            let config = config.unwrap();

            // println!("在标识头位置: {} 找到配置 {:?}", startSize,config);

            // 判断资源ID
            if config.Id.trim().eq(id) {
                // println!("符合当前匹配资源ID: {:?}", config);

                // 判断资源版本号是否支持当前版本
                let versionOrdering = compareVersion(&config.Version, &defaultResourceHead.Version);
                if versionOrdering.is_lt() {
                    return Err(Box::try_from(format!("Resource version does not match, the target resource version is {}, the current resource version is {}, please try to upgrade the program version", &config.Version, &defaultResourceHead.Version)).unwrap());
                }
                if versionOrdering.is_gt() {
                    return Err(Box::try_from(format!("Resource version does not match, the target resource version is {}, the current resource version is {}, please try to lower the program version", &config.Version, &defaultResourceHead.Version)).unwrap());
                }

                // 判断资源是否完整
                let resourceLength = config.Length.parse::<usize>().unwrap();
                sourceFile.seek(SeekFrom::Start((startSize + defaultResourceHead.getLen() as usize + resourceLength) as u64))?;
                let mut endBuffer: Vec<u8> = vec![0; END_IDENTIFIER.len()];
                sourceFile.read_exact(&mut *endBuffer)?;
                if !endBuffer.eq(&END_IDENTIFIER) {
                    return Err(Box::try_from("The resource to be exported is incomplete").unwrap());
                }

                // 写出文件(处理相对路径)
                let outputPath = if outputPath.is_relative() { targetFilePath.parent().unwrap().join(outputPath) } else { outputPath.to_path_buf() };
                let outputPath = if outputPath.is_dir() { outputPath.join(config.Name.trim()) } else { outputPath };
                let mut outputFile = File::create(&outputPath)?;

                sourceFile.seek(SeekFrom::Start((startSize + defaultResourceHead.getLen() as usize) as u64))?;

                // 循环读取并写出资源文件
                loop {
                    let nbytes = sourceFile.read(&mut buffer)?;

                    // 判断是否找到资源尾
                    // if let Some(size) = find_subsequence(&buffer, END_IDENTIFIER.as_ref()) {
                    if let Some(size) = memmem::Finder::new(END_IDENTIFIER.as_ref()).find(&buffer) {
                        outputFile.write_all(&buffer[..size])?;
                        break;
                    } else {
                        outputFile.write_all(&buffer[..nbytes])?;
                    }
                }

                // 处理压缩资源
                if config.Compress == CompressMode::Compress {
                    let actualFile = outputPath.parent().unwrap().join("actualFile");
                    decompressFile(&outputPath, &actualFile)?;
                    // 替换原资源文件
                    fs::remove_file(&outputPath)?;
                    fs::rename(actualFile, &outputPath)?;

                    // buffer = <[u8; 8192]>::try_from(decompress_to_vec(&buffer).unwrap()).unwrap();
                    // nbytes = buffer.len();
                    // println!("{:?}", data);
                }

                // 文件释放完成，检查文件大小
                if outputFile.metadata()?.len() != config.Size.trim().parse()? {
                    // 删除释放错误的文件
                    fs::remove_file(&outputPath)?;
                    return Err(Box::try_from("The resource to be exported is incomplete").unwrap());
                }

                // 文件释放成功
                return Ok(());
            }

            sourceFile.seek(SeekFrom::Start(oldSize))?;
        }

        // 如果文件搜寻完毕仍未找到资源信息
        if nbytes < &buffer.len() {
            return Err(Box::try_from("Resource not found").unwrap());
        }
    }
}

/// 寻找资源配置 - 从头至尾
/// # 参数
/// 1. 目标文件
/// 2. 回调函数(配置位置, 资源配置)
/// # 返回值
/// 资源配置 数组
pub fn _findResourcesConfig(targetFilePath: &Path, callback: fn(startSize: usize, config: &ResourceHead)) -> Result<Vec<ResourceHead>, Box<dyn Error>> {
    let defaultResourceHead = ResourceHead::default();

    // 打开目标文件
    let mut sourceFile = File::open(targetFilePath)?;

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut count = 0;

    let mut configs = Vec::new();

    loop {
        count += 1;
        let nbytes = &sourceFile.read(&mut buffer).unwrap();

        // 如果找到了资源
        // if let Some(size) = find_subsequence(&buffer, &*defaultResourceHead.getHead()) {
        if let Some(size) = memmem::Finder::new(&*defaultResourceHead.getHead()).find(&buffer) {
            // 资源文件头起始位置(我也不知道为什么要减8)
            let startSize = BUFFER_SIZE * (count - 1) + size - 8;
            let oldSize = sourceFile.stream_position().unwrap();

            // 偏移读取配置
            let mut configBuffer: Vec<u8> = vec![0; defaultResourceHead.getLen()];
            sourceFile.seek(SeekFrom::Start(startSize as u64))?;
            sourceFile.read_exact(&mut configBuffer)?;

            let config = ResourceHead::from(&configBuffer);
            if let Err(_e) = config {
                continue;
            }
            let config = config.unwrap();
            // println!("在标识头位置: {} 找到配置 {:?}", startSize,config);

            callback(startSize, &config);
            configs.push(config);
            sourceFile.seek(SeekFrom::Start(oldSize))?;
        }

        // 如果文件搜寻完毕
        if nbytes < &buffer.len() {
            break;
        }
    }
    Ok(configs)
}

/// 寻找字节（速度较慢）
/// # 返回值
/// 返回找到的字节位置
fn _find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// 比较版本号大小
fn compareVersion(version1: &str, version2: &str) -> Ordering {
    let nums1: Vec<&str> = version1.split('.').collect();
    let nums2: Vec<&str> = version2.split('.').collect();
    let n1 = nums1.len();
    let n2 = nums2.len();

    // 比较版本
    for i in 0..std::cmp::max(n1, n2) {
        let i1 = if i < n1 {
            nums1[i].parse::<i32>().unwrap()
        } else {
            0
        };
        let i2 = if i < n2 {
            nums2[i].parse::<i32>().unwrap()
        } else {
            0
        };
        if i1 != i2 {
            return if i1 > i2 {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
    }
    // 版本相等
    Ordering::Equal
}
