use crate::actionsapi::core::run_command;

/// Ham bir kabuk komutunu (shell command) sandbox içinde çalıştırır.
pub fn run_shell(command: &str) -> Result<(), String> {
    run_command("sh", &["-c", command])
}

/// Ham bir kabuk komutunu (shell command) sandbox içinde çalıştırır - alias.
pub fn run_shell_command(command: &str) -> Result<(), String> {
    run_shell(command)
}
