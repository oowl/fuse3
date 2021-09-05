use std::ffi::OsString;
#[cfg(target_os = "linux")]
use std::os::unix::io::RawFd;

#[cfg(target_os = "freebsd")]
use nix::mount::Nmount;
#[cfg(target_os = "linux")]
use nix::unistd;

/// mount options.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct MountOptions {
    // mount syscall data field option
    pub(crate) uid: Option<u32>,
    pub(crate) gid: Option<u32>,

    pub(crate) fs_name: Option<String>,

    // default 40000
    pub(crate) rootmode: Option<u32>,

    pub(crate) allow_root: bool,
    pub(crate) allow_other: bool,

    pub(crate) read_only: Option<bool>,

    // when run in privileged mode, it is lib self option
    pub(crate) nonempty: bool,

    // lib self option
    pub(crate) default_permissions: bool,

    pub(crate) dont_mask: bool,

    pub(crate) no_open_support: bool,
    pub(crate) no_open_dir_support: bool,

    pub(crate) handle_killpriv: bool,

    pub(crate) write_back: bool,

    pub(crate) force_readdir_plus: bool,

    pub(crate) custom_options: Option<OsString>,
}

impl MountOptions {
    /// set fuse filesystem mount `user_id`, default is current uid.
    pub fn uid(&mut self, uid: u32) -> &mut Self {
        self.uid.replace(uid);

        self
    }

    /// set fuse filesystem mount `group_id`, default is current gid.
    pub fn gid(&mut self, gid: u32) -> &mut Self {
        self.gid.replace(gid);

        self
    }

    /// set fuse filesystem name, default is **fuse**.
    pub fn fs_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.fs_name.replace(name.into());

        self
    }

    /// set fuse filesystem `rootmode`, default is 40000.
    pub fn rootmode(&mut self, rootmode: u32) -> &mut Self {
        self.rootmode.replace(rootmode);

        self
    }

    /// set fuse filesystem `allow_root` mount option, default is disable.
    pub fn allow_root(&mut self, allow_root: bool) -> &mut Self {
        self.allow_root = allow_root;

        self
    }

    /// set fuse filesystem `allow_other` mount option, default is disable.
    pub fn allow_other(&mut self, allow_other: bool) -> &mut Self {
        self.allow_other = allow_other;

        self
    }

    /// set fuse filesystem `ro` mount option, default is disable.
    pub fn read_only(&mut self, read_only: bool) -> &mut Self {
        self.read_only.replace(read_only);

        self
    }

    /// allow fuse filesystem mount on a non-empty directory, default is not allowed.
    pub fn nonempty(&mut self, nonempty: bool) -> &mut Self {
        self.nonempty = nonempty;

        self
    }

    /// set fuse filesystem `default_permissions` mount option, default is disable.
    ///
    /// When `default_permissions` is set, the [`raw::access`] and [`path::access`] is useless.
    ///
    /// [`raw::access`]: crate::raw::Filesystem::access
    /// [`path::access`]: crate::path::PathFilesystem::access
    pub fn default_permissions(&mut self, default_permissions: bool) -> &mut Self {
        self.default_permissions = default_permissions;

        self
    }

    /// don't apply umask to file mode on create operations, default is disable.
    pub fn dont_mask(&mut self, dont_mask: bool) -> &mut Self {
        self.dont_mask = dont_mask;

        self
    }

    /// make kernel support zero-message opens, default is disable
    pub fn no_open_support(&mut self, no_open_support: bool) -> &mut Self {
        self.no_open_support = no_open_support;

        self
    }

    /// make kernel support zero-message opendir, default is disable
    pub fn no_open_dir_support(&mut self, no_open_dir_support: bool) -> &mut Self {
        self.no_open_dir_support = no_open_dir_support;

        self
    }

    /// fs handle killing `suid`/`sgid`/`cap` on `write`/`chown`/`trunc`, default is disable.
    pub fn handle_killpriv(&mut self, handle_killpriv: bool) -> &mut Self {
        self.handle_killpriv = handle_killpriv;

        self
    }

    /// enable write back cache for buffered writes, default is disable.
    ///
    /// # Notes:
    ///
    /// if enable this feature, when write flags has `FUSE_WRITE_CACHE`, file handle is guessed.
    pub fn write_back(&mut self, write_back: bool) -> &mut Self {
        self.write_back = write_back;

        self
    }

    /// force filesystem use readdirplus only, when kernel use readdir will return `ENOSYS`,
    /// default is disable.
    ///
    /// # Notes:
    /// this may don't work with some old Linux Kernel.
    pub fn force_readdir_plus(&mut self, force_readdir_plus: bool) -> &mut Self {
        self.force_readdir_plus = force_readdir_plus;

        self
    }

    /// set custom options for fuse filesystem, the custom options will be used in mount
    pub fn custom_options(&mut self, custom_options: impl Into<OsString>) -> &mut Self {
        self.custom_options = Some(custom_options.into());

        self
    }

    #[cfg(target_os = "freebsd")]
    pub(crate) fn build(&self) -> Nmount {
        use cstr::cstr;

        let mut nmount = Nmount::new();
        nmount
            .str_opt(cstr!("fstype"), cstr!("fusefs"))
            .str_opt(cstr!("from"), cstr!("/dev/fuse"));
        if self.allow_other {
            nmount.null_opt(cstr!("allow_other"));
        }
        if self.default_permissions {
            nmount.null_opt(cstr!("default_permissions"));
        }
        if let Some(fs_name) = &self.fs_name {
            nmount.str_opt_owned(cstr!("subtype="), fs_name.as_str());
        }
        // TODO: additional options: push_symlinks_in, intr, max_read=, timeout=
        // TODO: mount flags like async, nosuid, noexec
        nmount
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn build(&self, fd: RawFd) -> OsString {
        let mut opts = vec![
            format!("fd={}", fd),
            format!(
                "user_id={}",
                self.uid.unwrap_or_else(|| unistd::getuid().as_raw())
            ),
            format!(
                "group_id={}",
                self.gid.unwrap_or_else(|| unistd::getgid().as_raw())
            ),
            format!("rootmode={}", self.rootmode.unwrap_or(40000)),
        ];

        if self.allow_root {
            opts.push("allow_root".to_string());
        }

        if self.allow_other {
            opts.push("allow_other".to_string());
        }

        if matches!(self.read_only, Some(true)) {
            opts.push("ro".to_string());
        }

        if self.default_permissions {
            opts.push("default_permissions".to_string());
        }

        let mut options = OsString::from(opts.join(","));

        if let Some(custom_options) = &self.custom_options {
            options.push(",");
            options.push(custom_options);
        }

        options
    }

    #[cfg(all(target_os = "linux", feature = "unprivileged"))]
    pub(crate) fn build_with_unprivileged(&self) -> OsString {
        let mut opts = vec![
            format!(
                "user_id={}",
                self.uid.unwrap_or_else(|| unistd::getuid().as_raw())
            ),
            format!(
                "group_id={}",
                self.gid.unwrap_or_else(|| unistd::getgid().as_raw())
            ),
            format!("rootmode={}", self.rootmode.unwrap_or(40000)),
            format!(
                "fsname={}",
                self.fs_name.as_ref().unwrap_or(&"fuse".to_string())
            ),
        ];

        if self.allow_root {
            opts.push("allow_root".to_string());
        }

        if self.allow_other {
            opts.push("allow_other".to_string());
        }

        if matches!(self.read_only, Some(true)) {
            opts.push("ro".to_string());
        }

        if self.default_permissions {
            opts.push("default_permissions".to_string());
        }

        let mut options = OsString::from(opts.join(","));

        if let Some(custom_options) = &self.custom_options {
            options.push(",");
            options.push(custom_options);
        }

        options
    }
}
