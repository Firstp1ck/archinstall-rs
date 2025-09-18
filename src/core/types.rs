use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Menu,
    Content,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Screen {
    Overview,
    Locales,
    MirrorsRepos,
    Disks,
    DiskEncryption,
    SwapPartition,
    Bootloader,
    UnifiedKernelImages,
    Hostname,
    RootPassword,
    UserAccount,
    ExperienceMode,
    Audio,
    Kernels,
    NetworkConfiguration,
    AdditionalPackages,
    Timezone,
    AutomaticTimeSync,
    SaveConfiguration,
    Install,
    Abort,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    KeyboardLayout,
    LocaleLanguage,
    LocaleEncoding,
    MirrorsRegions,
    OptionalRepos,
    MirrorsCustomServerInput,
    MirrorsCustomRepoName,
    MirrorsCustomRepoUrl,
    MirrorsCustomRepoSig,
    MirrorsCustomRepoSignOpt,
    DisksDeviceList,
    DiskEncryptionType,
    DiskEncryptionPassword,
    DiskEncryptionPasswordConfirm,
    DiskEncryptionPartitionList,
    HostnameInput,
    RootPassword,
    RootPasswordConfirm,
    UserAddUsername,
    UserAddPassword,
    UserAddPasswordConfirm,
    UserAddSudo,
    UserSelectEdit,
    UserSelectDelete,
    UserEditUsername,
    DesktopEnvSelect,
    ServerTypeSelect,
    XorgTypeSelect,
    Info,
    AbortConfirm,
    MinimalClearConfirm,
    KernelSelect,
    TimezoneSelect,
    AdditionalPackageInput,
    NetworkInterfaces,
    NetworkMode,
    NetworkIP,
    NetworkGateway,
    NetworkDNS,
    WipeConfirm,
    // Additional Packages: groups
    AdditionalPackageGroupSelect,
    AdditionalPackageGroupPackages,
}

#[derive(Clone)]
pub struct MenuEntry {
    pub label: String,
    pub content: String,
    pub screen: Screen,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepoSignature {
    Never,
    Optional,
    Required,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepoSignOption {
    TrustedOnly,
    TrustedAll,
}

#[derive(Clone)]
pub struct AdditionalPackage {
    pub name: String,
    pub repo: String,
    pub version: String,
    pub description: String,
}

#[derive(Clone)]
pub enum NetworkConfigMode {
    Dhcp,
    Static,
}

#[derive(Clone)]
pub struct NetworkInterfaceConfig {
    pub interface: String,
    pub mode: NetworkConfigMode,
    pub ip_cidr: Option<String>,
    pub gateway: Option<String>,
    pub dns: Option<String>,
}

#[derive(Clone)]
pub struct CustomRepo {
    pub name: String,
    pub url: String,
    pub signature: RepoSignature,
    pub sign_option: Option<RepoSignOption>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub password: String,
    pub password_hash: Option<String>,
    pub is_sudo: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DiskPartitionSpec {
    pub name: Option<String>,
    pub role: Option<String>,
    pub fs: Option<String>,
    pub start: Option<String>,
    pub size: Option<String>,
    pub flags: Vec<String>,
    pub mountpoint: Option<String>,
    pub mount_options: Option<String>,
    pub encrypt: Option<bool>,
}
