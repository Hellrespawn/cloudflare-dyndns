use camino::Utf8Path;

pub fn read_file_optional(path: &Utf8Path) -> Option<String> {
    fs_err::read_to_string(path).ok().map(|s| s.trim().to_owned())
}
