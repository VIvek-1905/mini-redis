use fuser::{FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::ENOENT;
use std::env;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(1);
const ENCRYPTION_KEY: u8 = 0xAA; // Our symmetric secret key

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1, size: 0, blocks: 0, atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
    kind: FileType::Directory, perm: 0o755, nlink: 2, uid: 501, gid: 20, rdev: 0, flags: 0, blksize: 512,
};

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2, size: 45, blocks: 1, atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
    kind: FileType::RegularFile, perm: 0o644, nlink: 1, uid: 501, gid: 20, rdev: 0, flags: 0, blksize: 512,
};

// The file system now holds state (the encrypted bytes)
struct HelloFS {
    encrypted_payload: Vec<u8>,
}

impl HelloFS {
    fn new() -> Self {
        let secret_text = "Highly classified data decrypted on the fly!\n";
        // Scramble the bytes in memory using the XOR cipher
        let scrambled: Vec<u8> = secret_text.bytes().map(|b| b ^ ENCRYPTION_KEY).collect();
        HelloFS { encrypted_payload: scrambled }
    }
}

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 && name.to_str() == Some("hello.txt") {
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "hello.txt"),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if ino == 2 {
            // Log the OS interaction to Terminal 1
            println!("OS requested read. Decrypting payload dynamically...");
            
            // Unscramble the payload back to readable text using the exact same key
            let decrypted: Vec<u8> = self.encrypted_payload.iter().map(|b| b ^ ENCRYPTION_KEY).collect();
            
            reply.data(&decrypted[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- <mount_point>");
        return;
    }
    
    let mountpoint = &args[1];
    let options = vec![MountOption::RO, MountOption::FSName("secure_vault".to_string())];
    
    println!("Mounting Encrypted Vault at: {}", mountpoint);
    println!("Press Ctrl+C to unmount and exit.");
    
    fuser::mount2(HelloFS::new(), mountpoint, &options).unwrap();
}