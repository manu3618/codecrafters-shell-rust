/// extract arguments from initial args
///
/// * split on space
/// * handle quotes
///
/// ```
/// use shell_starter_rust::*;
///
/// assert_eq!(parse_args("'shell hello'"), vec!["shell hello",]);
/// assert_eq!(parse_args("'world     test'"), vec!["world     test",]);
/// assert_eq!(
///     parse_args("'/tmp/file name' '/tmp/file name with spaces' "),
///     vec!["/tmp/file name", "/tmp/file name with spaces"]
/// );
/// assert_eq!(parse_args("''"), vec!["",]);
/// assert_eq!(parse_args(""), Vec::<String>::new());
/// assert_eq!(parse_args("a b c 'd '"), vec!["a", "b", "c", "d "]);
/// ```
///
pub fn parse_args(input: &str) -> Vec<String> {
    let input = &input.trim();
    if input.is_empty() {
        return Vec::new();
    }
    let quotes = ['"', '\''];
    if !input.contains(quotes) {
        return input.split_whitespace().map(String::from).collect();
    }

    for quote in quotes {
        if let Some(at) = input.find(quote) {
            let to = input[(at + 1)..]
                .find(quote)
                .expect("unable to find closing quote");
            let mut args = parse_args(&input[..at]);
            args.push(input[at + 1..at + to + 1].into());
            args.extend(parse_args(&input[(to + at + 2)..]));
            return args;
        }
    }
    unreachable!();
}
