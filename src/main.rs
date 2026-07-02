use mini_clap::Command;

fn main() {
    let cmd = Command::new("myapp");
    match cmd.parse(&[]).map(|m| m.command_name) {
        Ok(name) => println!("Command: {name}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
