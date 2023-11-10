use anyhow::anyhow;
use anyhow::Result;
use log::warn;
use std::path::Component;
use std::{
    collections::HashMap,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};

use buhao_lib::{Contents, DirectoryContents, DirectoryItem, Inode, InodeId, INVALID_PARENT};

pub struct Filesystem {
    root_path: PathBuf,
    root: InodeId,
    inodes: HashMap<InodeId, Inode>,
}

impl Filesystem {
    pub fn load_from_fs(root_path: &Path) -> Self {
        let root_metadata = std::fs::metadata(root_path).unwrap();
        let root = root_metadata.ino();
        let inodes = HashMap::new();
        let mut fs = Self {
            root_path: root_path.to_path_buf(),
            root,
            inodes,
        };
        let root_files = dfs_list(&mut fs, root_path).unwrap();
        fs.update(Inode::new(
            root_metadata,
            Contents::Directory(DirectoryContents {
                parent: INVALID_PARENT,
                children: root_files,
            }),
        ));
        fs
    }

    pub fn update(&mut self, inode: Inode) {
        self.inodes.insert(inode.id, inode);
    }

    pub fn open(&self, path: &Path) -> Result<&Inode> {
        // invariant: inode is always a directory until the end of the loop
        let mut inode = self.inodes.get(&self.root).unwrap();
        let relative = if path.is_absolute() {
            path.strip_prefix(&self.root_path)
                .map_err(|x| anyhow!("Unmanaged path: {}", x))?
        } else {
            path
        };
        for component in relative.components() {
            match component {
                Component::Prefix(_) => unreachable!(),
                Component::RootDir => continue,
                Component::CurDir => continue,
                Component::ParentDir => {
                    let parent_id = match inode.contents {
                        Contents::Directory(ref contents) => contents.parent,
                        _ => unreachable!(),
                    };
                    if parent_id == INVALID_PARENT {
                        return Err(anyhow!("Invalid path: {}", path.display()));
                    }
                    inode = self.inodes.get(&parent_id).unwrap();
                }
                Component::Normal(name) => {
                    let directory = match inode.contents {
                        Contents::Directory(ref contents) => contents,
                        _ => unreachable!(),
                    };
                    let mut found = false;
                    for item in &directory.children {
                        if item.name == name.to_string_lossy() {
                            found = true;
                            inode = self.inodes.get(&item.inode).unwrap();
                            break;
                        }
                    }
                    if !found {
                        return Err(anyhow!("Invalid path: {}", path.display()));
                    }
                }
            }
        }
        Ok(inode)
    }
}

pub fn dfs_list(filesystem: &mut Filesystem, dir: &Path) -> Result<Vec<DirectoryItem>> {
    let self_id = std::fs::metadata(dir)?.ino();
    let paths = std::fs::read_dir(dir)?;
    let mut items = Vec::new();
    for path in paths {
        let path = match path {
            Ok(path) => path,
            Err(ref e) => {
                warn!("Failed to read directory {:?} entry: {}", path, e);
                continue;
            }
        };
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(ref e) => {
                warn!("Failed to read {:?} metadata: {}", path, e);
                continue;
            }
        };
        let filetype = metadata.file_type();
        let contents = {
            if filetype.is_symlink() {
                let target = match std::fs::read_link(path.path()) {
                    Ok(target) => target,
                    Err(ref e) => {
                        warn!("Failed to read symlink {:?} target: {}", path, e);
                        continue;
                    }
                };
                Contents::Symlink(target.to_string_lossy().to_string())
            } else if filetype.is_file() {
                Contents::File
            } else if filetype.is_dir() {
                let children = match dfs_list(filesystem, path.path().as_path()) {
                    Ok(children) => children,
                    Err(ref e) => {
                        warn!("Failed to read directory {:?} contents: {}", path, e);
                        continue;
                    }
                };
                Contents::Directory(DirectoryContents {
                    parent: self_id,
                    children,
                })
            } else {
                continue;
            }
        };
        let id = metadata.ino();
        items.push(DirectoryItem {
            name: path.file_name().as_os_str().to_string_lossy().to_string(),
            inode: id,
        });
        filesystem.update(Inode::new(metadata, contents));
    }

    Ok(items)
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write, os::unix::fs::symlink};

    use super::*;
    use test_log::test;

    /// Setup /tmp/buhao
    /// ├── a
    /// └── b
    ///     └── c -> ../a
    #[ctor::ctor]
    fn setup() {
        std::fs::create_dir_all("/tmp/buhao").unwrap();
        let mut a = File::create("/tmp/buhao/a").unwrap();
        a.write_all(b"hello").unwrap();
        std::fs::create_dir_all("/tmp/buhao/b").unwrap();

        // remove symlink if exists
        std::fs::remove_file("/tmp/buhao/b/c").unwrap_or(());
        symlink("../a", "/tmp/buhao/b/c").unwrap();
    }

    #[test]
    fn test_open() {
        let filesystem = Filesystem::load_from_fs(Path::new("/tmp/buhao"));
        println!(
            "root path: {:?}, root: {}",
            filesystem.root_path, filesystem.root
        );
        for item in filesystem.inodes.values() {
            println!("{:?}", item);
        }
        let inode = filesystem.open(Path::new("./a")).unwrap();
        println!("./a: {:?}", inode);
        let inode = filesystem.open(Path::new("/tmp/buhao/a")).unwrap();
        println!("/tmp/buhao/a: {:?}", inode);

        let inode = filesystem.open(Path::new("./b")).unwrap();
        println!("./b: {:?}", inode);

        let inode = filesystem.open(Path::new("./b/c")).unwrap();
        println!("./b/c: {:?}", inode);
    }
}
