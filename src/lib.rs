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
/// assert_eq!(parse_args("\"before\\  after\""), vec![r"before\  after"]);
/// assert_eq!(parse_args(r"world\ \ \ \ \ \ script"), vec!["world      script"]);
/// assert_eq!(
///     parse_args("\"/tmp/file\\name\" \"/tmp/file\\ name\""),
///     vec![r"/tmp/file\name", r"/tmp/file\ name"]
/// );
/// assert_eq!(parse_args("world     test"), vec!["world", "test"]);
/// assert_eq!(parse_args(r"'shell\\\nscript'"), vec![r"shell\\\nscript"]);
/// assert_eq!(
///     parse_args("'example\\\"testhello\\\"shell'"),
///     vec!["example\\\"testhello\\\"shell"]
/// );
/// assert_eq!(parse_args("world     test"), vec!["world", "test"]);
/// assert_eq!(parse_args("world     test"), vec!["world", "test"]);
/// ```
///
pub fn parse_args(input: &str) -> Vec<String> {
    let input = &input.trim();
    if input.is_empty() {
        return Vec::new();
    }
    let mut  quotes = ['"', '\''];
    if !input.contains(quotes) {
        let mut res = Vec::new();
        let mut buff = String::new();
        let mut escaping = false;
        for c in input.chars() {
            if escaping {
                buff.push(c.clone());
                escaping = false;
                continue;
            }
            match c {
                '\\' => escaping = true,
                l if l.is_whitespace() => {
                    if !buff.trim().is_empty() {
                        res.push(String::from(buff.trim()).clone());
                    }
                    buff.clear();
                }
                _ => buff.push(c.into()),
            }
        }
        if !buff.is_empty() {
            res.push(String::from(buff.trim()).clone());
        }
        return res;
    }

    // handle first encontered quote first
    quotes.sort_by_key(|&k| input.find(k).unwrap_or(input.len()));
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
