use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

pub fn set_var<K: AsRef<str>, V: AsRef<str>>(key: K, val: V) {
    let _guard = ENV_LOCK.lock().expect("env lock poisoned");
    unsafe { std::env::set_var(key.as_ref(), val.as_ref()); }
}

pub fn remove_var<K: AsRef<str>>(key: K) {
    let _guard = ENV_LOCK.lock().expect("env lock poisoned");
    unsafe { std::env::remove_var(key.as_ref()); }
}
