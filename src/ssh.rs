use std::process::Command;
use rpassword::read_password;

pub fn ssh_connect(username: &str) {
    // Extract cluster name (everything before first underscore)
    let cluster = username.split('_').next().unwrap_or("");

    // Prompt for password securely (no echo)
    println!("Enter password for {}:", username);
    let password = match read_password() {
        Ok(pw) => pw,
        Err(e) => {
            eprintln!("Failed to read password: {}", e);
            return;
        }
    };

    let ssh_command = format!(
        "sshpass -p '{}' ssh -o StrictHostKeyChecking=no {}@ssh.{}.service.one",
        password, username, cluster
    );

    //println!("Running SSH command...");

    let _status = Command::new("sh")
        .arg("-c")
        .arg(&ssh_command)
        .status();

}
