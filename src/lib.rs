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
/// assert_eq!(parse_args("\"hello'script'\\\\n'world\""), vec![r"hello'script'\n'world"]);
/// assert_eq!(parse_args("\"example\\\"insidequotes\"hello\\\""), vec!["example\"insidequoteshello\""]);
/// assert_eq!(parse_args("\"example\\\"inside\"test\\\""), vec!["example\"insidetest\""]);
/// assert_eq!(parse_args("\"mixed\\\"quote\'example'\\\\\""), vec!["mixed\"quote\'example\'\\"]);
/// ```
///
pub fn parse_args(input: &str) -> Vec<String> {
    let input = &input.trim();
    if input.is_empty() {
        return Vec::new();
    }
    let mut args = Vec::new();
    let mut buff = String::new();
    let mut quotes = ['"', '\''];
    let mut escaping = false;
    quotes.sort_by_key(|&k| input.find(k).unwrap_or(input.len()));
    let mut cursor = 0;
    while cursor <= input.len() {
        let c = match input.chars().nth(cursor) {
            Some(x) => x,
            None => break,
        };
        if escaping {
            buff.push(c);
            escaping = false;
            cursor += 1;
            continue;
        }
        match c {
            '"' => {
                let quote_idx = find_quote(&input[cursor..], '"');
                let at = cursor + quote_idx.first().expect("Already handle empty vector");
                let to = cursor + quote_idx.get(1).expect("unable to find closing quote");
                buff += &handle_double_quoted(&input[at + 1..to]);
                cursor = to;
            }
            '\'' => {
                let quote_idx = find_quote(&input[cursor..], '\'');
                let at = cursor + quote_idx.first().expect("Alredy handle empty vector");
                let to = cursor + quote_idx.get(1).expect("unable to find closing quote");
                buff += &input[at + 1..to];
                cursor = to;
            }
            '\\' => escaping = true,
            l if l.is_whitespace() => {
                if !buff.trim().is_empty() {
                    args.push(String::from(buff.trim()).clone());
                    buff.clear();
                };
            }
            _ => buff.push(c),
        }
        cursor += 1;
    }
    args.push(buff);
    args
}

fn find_quote(input: &str, quote: char) -> Vec<usize> {
    match quote {
        '"' => find_unescaped_doublequotes(input),
        _ => input.match_indices(quote).map(|(idx, _)| idx).collect(),
    }
}

fn find_unescaped_doublequotes(input: &str) -> Vec<usize> {
    let mut res = Vec::new();
    let mut escaping = false;
    for (idx, c) in input.chars().enumerate() {
        if escaping {
            escaping = false;
        } else {
            match c {
                '\\' => escaping = true,
                '"' => res.push(idx),
                _ => continue,
            }
        }
    }
    res
}

/// Inside double quotes, some backslash are escape characters
fn handle_double_quoted(input: &str) -> String {
    let mut res = String::new();
    let mut escaping = false;
    let mut inside_quote = false;
    for c in input.chars() {
        if inside_quote {
            if c == '\'' {
                inside_quote = false;
            }
            res.push(c);
        } else if escaping {
            match c {
                '\\' | '$' | '"' => res.push(c),
                _ => res += format!("\\{}", c).as_str(),
            }
            escaping = false;
        } else {
            match c {
                '\\' => {
                    escaping = true;
                    continue;
                }
                '\'' => {
                    inside_quote = true;
                    res.push(c);
                }
                _ => res.push(c),
            }
        }
    }
    res
}
