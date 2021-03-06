use std::fs::File;
use std::io::{Write, Read};
use std::io;
use std::path::PathBuf;
use zip::{ZipWriter, write, CompressionMethod};
use zip::result::ZipResult;
use zip::write::FileOptions;
use crate::file_utils::make_relative_path;

/// Creates a zip archive that contains the files and directories from the specified directory.
pub fn zip_create_from_directory(archive_file: &PathBuf, directory: &PathBuf) -> ZipResult<()> {
    let options = write::FileOptions::default().compression_method(CompressionMethod::Stored);
    zip_create_from_directory_with_options(archive_file, directory, options)
}

/// Creates a zip archive that contains the files and directories from the specified directory, uses the specified compression level.
pub fn zip_create_from_directory_with_options(archive_file: &PathBuf, directory: &PathBuf, options: FileOptions) -> ZipResult<()> {
    let file = File::create(archive_file).unwrap();
    let mut zip_writer = zip::ZipWriter::new(file);
    zip_writer.create_from_directory_with_options(directory, options)
}

pub trait ZipWriterExtensions {
    /// Creates a zip archive that contains the files and directories from the specified directory.
    fn create_from_directory(&mut self, directory: &PathBuf) -> ZipResult<()>;

    /// Creates a zip archive that contains the files and directories from the specified directory, uses the specified compression level.
    fn create_from_directory_with_options(&mut self, directory: &PathBuf, options: FileOptions) -> ZipResult<()>;
}

impl<W: Write + io::Seek> ZipWriterExtensions for ZipWriter<W> {
    fn create_from_directory(&mut self, directory: &PathBuf) -> ZipResult<()> {
        let options = write::FileOptions::default().compression_method(CompressionMethod::Stored);
        self.create_from_directory_with_options(directory, options)
    }

    fn create_from_directory_with_options(&mut self, directory: &PathBuf, options: FileOptions) -> ZipResult<()> {
        let mut paths_queue: Vec<PathBuf> = vec![];
        paths_queue.push(directory.clone());

        let mut buffer = Vec::new();

        while paths_queue.len() > 0 {
            let next = paths_queue.pop().unwrap();
            let directory_entry_iterator = std::fs::read_dir(next).unwrap();

            for entry in directory_entry_iterator {
                let entry_path = entry.unwrap().path();
                let entry_metadata = std::fs::metadata(entry_path.clone()).unwrap();
                if entry_metadata.is_file() {
                    let mut f = File::open(&entry_path).unwrap();
                    f.read_to_end(&mut buffer).unwrap();
                    let relative_path = make_relative_path(&directory, &entry_path);
                    self.start_file_from_path(&relative_path, options).unwrap();
                    self.write_all(buffer.as_ref()).unwrap();
                    buffer.clear();
                } else if entry_metadata.is_dir() {
                    let relative_path = make_relative_path(&directory, &entry_path);
                    self.add_directory_from_path(&relative_path, options).unwrap();
                    paths_queue.push(entry_path.clone());
                }
            }
        }

        self.finish().unwrap();
        Ok(())
    }
}