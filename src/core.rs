use std::path::Path;
use std::error::Error;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, SeekFrom, Seek};
use crate::util::{find_subsequence, compareVersion, compressionFile};
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;
use std::fs;

/// 缓冲区大小（8KB）
pub const BUFFER_SIZE: usize = 1024 * 8;

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
pub fn addResource(targetFilePath: &Path, sourceFilePath: &Path, id: &str, compressionGrade: Option<u8>, outputPath: Option<&Path>) -> Result<(), Box<dyn Error>> {
    // 打开资源文件
    let sourceFilePath = if sourceFilePath.is_relative() { targetFilePath.parent().unwrap().join(sourceFilePath) } else { sourceFilePath.to_path_buf() };
    let mut sourceFile = File::open(&sourceFilePath)?;
    let sourceName = &sourceFilePath.file_name().unwrap().to_str().unwrap();

    if let Some(grage) = compressionGrade {
        let compressFile = &*sourceFilePath.parent().unwrap().join("temp");
        compressionFile(&*sourceFilePath, compressFile, grage)?;
        sourceFile = File::open(&compressFile)?;
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
    let head = ResourceHead::new(id, sourceLength,sourceLength, sourceName, compressMode).to_bytes()?;
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
        if let Some(size) = find_subsequence(&buffer, &*defaultResourceHead.getHead()) {
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
                let mut outputFile = File::create(outputPath)?;

                sourceFile.seek(SeekFrom::Start((startSize + defaultResourceHead.getLen() as usize) as u64))?;

                // 循环读取并写出资源文件
                loop {
                    let nbytes = sourceFile.read(&mut buffer)?;
                    if config.Compress == CompressMode::Compress {
                        // buffer = <[u8; 8192]>::try_from(decompress_to_vec(&buffer).unwrap()).unwrap();
                        // nbytes = buffer.len();
                        // println!("{:?}", data);
                    }

                    // 判断是否找到资源尾
                    if let Some(size) = find_subsequence(&buffer, END_IDENTIFIER.as_ref()) {
                        outputFile.write_all(&buffer[..size])?;
                        break;
                    } else {
                        outputFile.write_all(&buffer[..nbytes])?;
                    }
                }

                // 文件释放完成，检查文件大小
                if outputFile.metadata()?.len() != config.Size.trim().parse()? {
                    return Err(Box::try_from("The resource to be exported is incomplete").unwrap());
                }

                // 文件释放成功
                return Ok(());
            }

            sourceFile.seek(SeekFrom::Start(oldSize))?;
        }

        // 如果文件搜寻完毕
        if nbytes < &buffer.len() {
            return Err(Box::try_from("Resource not found").unwrap());
        }
    }
}


/// 获取指定文件的资源列表
pub fn getResourceList(targetFilePath: &Path) -> Result<Vec<ResourceHead>, Box<dyn Error>> {
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
        if let Some(size) = find_subsequence(&buffer, &*defaultResourceHead.getHead()) {
            // 资源文件头起始位置(我也不知道为什么要减8)
            let startSize = BUFFER_SIZE * (count - 1) + size - 8;

            let oldSize = sourceFile.stream_position().unwrap();

            // 偏移读取配置
            let mut configBuffer: Vec<u8> = vec![0; defaultResourceHead.getLen()];
            sourceFile.seek(SeekFrom::Start(startSize as u64))?;
            sourceFile.read_exact(&mut configBuffer)?;

            // 解析配置
            let config = ResourceHead::from(&configBuffer)?;
            configs.push(config);

            sourceFile.seek(SeekFrom::Start(oldSize))?;
        }

        // 如果文件搜寻完毕
        if nbytes < &buffer.len() { break; }
    }
    Ok(configs)
}
