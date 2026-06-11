//! Shell-specific services that run alongside the UI.

pub mod idle_inhibit;
pub mod shell_ipc;

pub use idle_inhibit::IdleInhibitService;
pub use shell_ipc::ShellIpcService;
