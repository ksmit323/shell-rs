use std::path::Path;

pub fn find_command(command: &str, paths: &[String]) -> Option<String> {
    if command.contains('/') {
        return find_absolute_command(command);
    }
    find_command_in_path(command, paths)
}

fn find_absolute_command(command: &str) -> Option<String> {
    let path = Path::new(command);
    path.exists().then(|| command.to_string())
}

fn find_command_in_path(command: &str, paths: &[String]) -> Option<String> {
    paths.iter()
        .map(|path_dir| Path::new(path_dir).join(command))
        .find(|path| path.exists())
        .map(|path| path.to_string_lossy().into_owned())
}