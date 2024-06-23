use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{Error as IoError, Read, Write},
    path::{Path, PathBuf},
};

use crate::{Font, RenderingContext, Texture};

pub type BufReader = std::io::BufReader<File>;
pub type BufWriter = std::io::BufWriter<File>;

// Debug: expect working dir to be cargo project, so look for assets relative to that
#[cfg(debug_assertions)]
pub fn base_path() -> PathBuf {
    PathBuf::new()
}

// Release: always look for assets relative to the executable
#[cfg(not(debug_assertions))]
pub fn base_path() -> PathBuf {
    // TODO cache this
    let mut dir = env::current_exe().unwrap();
    dir.pop();
    dir
}

#[derive(Debug)]
enum ErrorKind {
    IoError(IoError),
    Format,
    Other,
}

pub struct AssetError {
    path: PathBuf,
    write: bool,
    kind: ErrorKind,
    info: String,
}

impl AssetError {
    fn new_io(path: PathBuf, write: bool, error: IoError) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::IoError(error),
            info: String::new(),
        }
    }
    fn new_yaml(path: PathBuf, write: bool, error: serde_yml::Error) -> Self {
        AssetError {
            path,
            write,
            kind: ErrorKind::Format,
            info: error.to_string(),
        }
    }
    fn new_png(path: PathBuf, error: png::DecodingError) -> Self {
        match error {
            png::DecodingError::IoError(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::IoError(error),
                info: String::new(),
            },
            png::DecodingError::Format(error) => AssetError {
                path,
                write: false,
                kind: ErrorKind::Format,
                info: error.to_string(),
            },
            _ => AssetError {
                path,
                write: false,
                kind: ErrorKind::Other,
                info: error.to_string(),
            },
        }
    }

    pub fn not_found(&self) -> bool {
        if let ErrorKind::IoError(error) = &self.kind {
            error.kind() == std::io::ErrorKind::NotFound
        } else {
            false
        }
    }
}

impl std::fmt::Debug for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let read_write = if self.write { "writing" } else { "reading" };
        write!(f, "Error {} {}: ", read_write, self.path.to_string_lossy())?;
        if let ErrorKind::IoError(error) = &self.kind {
            if self.write && error.kind() == std::io::ErrorKind::NotFound {
                // When a NotFound error occurs while writing, it means a parent directory doesn't exist.
                write!(f, "The parent directory does not exist.")?;
                if let Some(code) = error.raw_os_error() {
                    write!(f, " (os error {code})")?;
                }
                Ok(())
            } else {
                write!(f, "{}", error)
            }
        } else {
            write!(f, "{}", self.info)
        }
    }
}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for AssetError {}

pub type Result<T> = std::result::Result<T, AssetError>;

pub fn get_path(prefix: &str, asset_path: &str) -> PathBuf {
    let mut file_path = base_path();
    file_path.push(prefix);
    file_path.push(asset_path);
    file_path
}
pub fn open_reader(path: &Path) -> Result<BufReader> {
    println!("Reading {}", path.to_string_lossy());
    let file = File::open(path).map_err(|e| AssetError::new_io(path.to_owned(), false, e))?;
    Ok(BufReader::new(file))
}
pub fn open_writer(path: &Path) -> Result<BufWriter> {
    println!("Writing {}", path.to_string_lossy());
    let file = File::create(path).map_err(|e| AssetError::new_io(path.to_owned(), true, e))?;
    Ok(BufWriter::new(file))
}

pub fn create_dir(dir: &str) {
    let mut dir_path = base_path();
    dir_path.push(dir);
    if !dir_path.exists() {
        println!("Creating directory {}", dir_path.to_string_lossy());
        std::fs::create_dir(dir_path).expect("could not create directory");
    }
}

pub fn load_text_file(prefix: &str, file: &str) -> Result<String> {
    let path = get_path(prefix, file);
    let mut reader = open_reader(&path)?;
    let mut string = String::new();
    reader
        .read_to_string(&mut string)
        .map_err(|e| AssetError::new_io(path, false, e))?;
    Ok(string)
}
pub fn save_text_file(prefix: &str, file: &str, value: &str) -> Result<()> {
    let path = get_path(prefix, file);
    let mut writer = open_writer(&path)?;
    writer
        .write_all(value.as_bytes())
        .map_err(|e| AssetError::new_io(path, true, e))?;
    Ok(())
}

pub fn load_yaml_file<T>(prefix: &str, file: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let path = get_path(prefix, file);
    let reader = open_reader(&path)?;
    serde_yml::from_reader(reader).map_err(|e| AssetError::new_yaml(path, false, e))
}
pub fn load_yaml_file_or_default<T>(prefix: &str, file: &str) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let path = get_path(prefix, file);
    if path.exists() {
        let reader = open_reader(&path)?;
        serde_yml::from_reader(reader).map_err(|e| AssetError::new_yaml(path, false, e))
    } else {
        println!(
            "{} does not exist, using defaults instead",
            path.to_string_lossy()
        );
        Ok(Default::default())
    }
}
pub fn save_yaml_file<T>(prefix: &str, file: &str, value: &T) -> Result<()>
where
    T: Serialize,
{
    let path = get_path(prefix, file);
    let writer = open_writer(&path)?;
    serde_yml::to_writer(writer, value).map_err(|e| AssetError::new_yaml(path, true, e))
}

pub fn load_png_file(context: &mut RenderingContext, prefix: &str, file: &str) -> Result<Texture> {
    let path = get_path(prefix, file);
    let reader = open_reader(&path)?;
    let decoder = png::Decoder::new(reader);
    let mut image_reader = decoder
        .read_info()
        .map_err(|e| AssetError::new_png(path.clone(), e))?;
    let mut buffer = vec![0; image_reader.output_buffer_size()];
    let info = image_reader
        .next_frame(&mut buffer)
        .map_err(|e| AssetError::new_png(path, e))?;
    buffer.truncate(info.buffer_size());
    Ok(Texture::new_rgba8(
        context,
        info.width.try_into().unwrap(),
        info.height.try_into().unwrap(),
        &buffer,
    ))
}

pub fn load_font_file(prefix: &str, file: &str) -> Result<Font> {
    let path = get_path(prefix, file);
    let mut reader = open_reader(&path)?;
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .map_err(|e| AssetError::new_io(path.clone(), false, e))?;
    Font::try_from_vec(buf).map_err(|_e| AssetError {
        path,
        write: false,
        kind: ErrorKind::Format,
        info: "Invalid font".to_string(),
    })
}
