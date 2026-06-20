use clap::Subcommand;

/// Window switcher control subcommands.
#[derive(Subcommand, Debug)]
pub enum WindowCommands {
    /// Advance the window switcher's highlighted selection, opening it if
    /// closed. Intended to be bound to a repeating key (e.g. Tab while a
    /// modifier is held), with `cycle-commit` bound to the modifier's
    /// release.
    CycleStep,

    /// Activate the window switcher's currently highlighted selection and
    /// close it.
    CycleCommit,
}
