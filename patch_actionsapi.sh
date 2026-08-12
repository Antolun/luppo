sed -i '/let mut child = Command::new("sh")/i \
    println!("DEBUG RUN_COMMAND: name={}, cwd={:?}, PATH={:?}", name, std::env::current_dir(), std::env::var("PATH"));' pisi-builder/src/actionsapi.rs
