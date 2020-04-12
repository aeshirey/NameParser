mod config;
use config::*;

// For appending text to a PersonName field
macro_rules! append_part {
    ($part:expr, $ex:expr) => {{
        if !$part.is_empty() {
            $part.push(' ');
        }

        $part.push_str($ex);
    }};
}

#[derive(Debug, PartialEq)]
pub struct PersonName {
    pub title: String,
    pub first: String,
    pub middle: String,
    pub last: String,
    pub suffix: String,
    pub nicknames: Vec<String>,
}

impl PersonName {
    /// Returns a new, empty `PersonName`. Used primarily for `PersonName::parse`.
    pub fn new() -> Self {
        PersonName {
            title: "".to_string(),
            first: "".to_string(),
            middle: "".to_string(),
            last: "".to_string(),
            suffix: "".to_string(),
            nicknames: Vec::new(),
        }
    }

    /// Parses a `&str` into a `PersonName`. If the value can't be reasonably parsed (eg, because
    /// it's the empty string), `None` is returned. Otherwise, a `PersonName` is returned with as
    /// many fields as can be reasonably extracted.
    pub fn parse(full_name: &str) -> Option<Self> {
        let mut person = PersonName::new();

        let (full_name, nicknames) = extract_nicknames(full_name);
        person.nicknames = nicknames;

        let csv_parts: Vec<&str> = full_name
            .split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect();

        if csv_parts.is_empty() {
            return if !person.nicknames.is_empty() {
                // nickname only
                Some(person)
            } else {
                // Can't parse the name/nothing there
                None
            };
        }

        if csv_parts.len() == 1 {
            // Format assumed to be: "title first middle middle middle last suffix"
            let part = csv_parts[0];

            // preprocess_input will join on conjunctions and add any joined titles to person.title
            let pieces: Vec<String> = person.preprocess_input(part);

            if pieces.len() == 1 && is_title(&pieces[0]) {
                // exactly one part, and it's a title
                append_part!(person.title, &&pieces[0][..])
            } else {
                for i in 0..pieces.len() {
                    match &pieces[i..] {
                        [c, ..] if is_title(&c) => {
                            if !person.first.is_empty() || !person.middle.is_empty() {
                                append_part!(person.last, c)
                            } else {
                                append_part!(person.title, c)
                            }
                        }
                        [c, ..] if person.first.is_empty() => append_part!(person.first, c),
                        [c, end @ ..] if end.iter().all(|part| is_suffix(&part)) => {
                            // last name plus one or more suffixes
                            append_part!(person.last, c);
                            end.iter().for_each(|suf| append_part!(person.suffix, suf));
                            break;
                        }
                        [c, _n, ..] if is_prefix(c) => {
                            // This part looks like a prefix to a last name, so put it there
                            append_part!(person.last, c);
                        }
                        [c, _n, ..] => {
                            // another component exists, so this is likely a middle name
                            append_part!(person.middle, c);
                        }
                        [c] if !person.last.is_empty() && is_suffix(&c) => {
                            append_part!(person.last, c)
                        }
                        [c] => append_part!(person.last, c),
                        [] => {}
                    };
                }
            }
        } else if csv_parts[1].split(' ').all(|part| is_suffix(part)) {
            // title first middle last [suffix], suffix
            // csv_parts[0],                     csv_parts[1:...]
            append_part!(person.suffix, csv_parts[1]);
            let pieces: Vec<String> = person.preprocess_input(csv_parts[0]);

            if pieces.len() == 1 && is_title(&pieces[0]) {
                // exactly one part, and it's a title
                append_part!(person.title, &&pieces[0][..])
            } else {
                for i in 0..pieces.len() {
                    match &pieces[i..] {
                        [c, ..] if is_title(&c) => append_part!(person.title, c),
                        [c, ..] if person.first.is_empty() => append_part!(person.first, c),
                        [c, end @ ..] if end.iter().all(|part| is_suffix(&part)) => {
                            append_part!(person.last, c);
                            end.iter()
                                .for_each(|part| append_part!(person.suffix, part));
                            break;
                        }
                        [c, _n, ..] if is_prefix(&c) => append_part!(person.last, c), // another component exists, so this is likely a middle name
                        [c, _n, ..] => append_part!(person.middle, c), // another component exists, so this is likely a middle name
                        [c] => append_part!(person.last, c),
                        [] => {}
                    }
                }
            }
        } else {
            // last [suffix], title first middles[,] suffix [,suffix]
            // csv_parts[0],  csv_parts[1],          csv_parts[2..]

            // lastname part may have suffixes in it
            let last_name_pieces = person.preprocess_input(csv_parts[0]);
            for piece in &last_name_pieces {
                if is_suffix(&piece[..]) && !person.last.is_empty() {
                    append_part!(person.suffix, piece)
                } else {
                    append_part!(person.last, piece)
                }
            }

            let pieces = person.preprocess_input(csv_parts[1]);

            if pieces.len() == 1 && is_title(&pieces[0]) {
                append_part!(person.first, &&pieces[0][..]);
            } else {
                for i in 0..pieces.len() {
                    match &pieces[i..] {
                        [c, ..] if is_title(&c) => append_part!(person.title, c),
                        [c, ..] if person.first.is_empty() => append_part!(person.first, c),
                        [c, ..] if is_suffix(c) => append_part!(person.suffix, c),
                        [c, ..] if is_prefix(&c) => append_part!(person.middle, c),
                        [c, ..] => append_part!(person.middle, c),
                        [] => {}
                    }
                }
            }

            csv_parts[2..]
                .iter()
                .for_each(|part| append_part!(person.suffix, &part));
        }

        person.postprocess_first();

        Some(person)
    }

    /// Postprocess the first name to handle post-parsing corrections.
    fn postprocess_first(&mut self) {
        // For input like "Mr. Johnson", we'll generally have the correct title (Mr.) but the first
        // will be Johnson and last is empty.
        if !self.title.is_empty() && !self.first.is_empty() && self.last.is_empty() {
            self.last = self.first.to_string();
            self.first.clear();
        }
    }

    /// Tokenize a string, joining tokens on conjunctions (eg, "Mr.", "and", "Mrs." becomes "Mr. and
    /// Mrs.").
    // TODO: Include last name prefixes here (see https://en.wikipedia.org/wiki/Tussenvoegsel)
    fn preprocess_input(&mut self, parts: &str) -> Vec<String> {
        let tmp: Vec<String> = parts
            .split(' ')
            .filter(|part| !part.is_empty())
            .map(|part| part.to_string())
            .collect();

        self.join_on_conjunctions(&tmp)
    }

    /// Performs the joining of tokens on conjunctions
    fn join_on_conjunctions(&mut self, parts: &Vec<String>) -> Vec<String> {
        let conjunction = (1..parts.len() - 1).find(|i| {
            let part: &str = &&parts[*i as usize][..];
            CONJUNCTIONS.contains(&part)
        });

        if let Some(i) = conjunction {
            let joined = parts[i - 1..=i + 1].join(" ");
            let prev = parts.get(i - 1).unwrap();

            // if we're joining titles ('mr', 'and', 'mrs'), then we'll save this
            if is_title(&prev) {
                if !self.title.is_empty() {
                    self.title.push(' ');
                }
                self.title.push_str(&joined);
            }

            let acc = parts
                .iter()
                .take(i - 1)
                .chain(&[joined])
                .chain(parts.iter().skip(i + 2))
                .map(|s| s.to_string())
                .collect();

            self.join_on_conjunctions(&acc)
        } else {
            parts.to_vec()
        }
    }
}

/// Extract nicknames from the input, returning the original string without nicknames plus a list
/// of nicknames extracted, in-order.
fn extract_nicknames(full_name: &str) -> (String, Vec<String>) {
    let mut nicks = Vec::new();
    let mut name_without_nicks = full_name.to_string();

    // TODO: handle ' and " such as: James "Jimmy" Carter, but do them in order.
    while let Some((opos, cpos)) = find_nickname_start_end(&name_without_nicks, '(', ')') {
        // get the nickname
        nicks.push(name_without_nicks[opos + 1..cpos].to_string());
        name_without_nicks = name_without_nicks[..opos].trim_end().to_string()
            + " "
            + name_without_nicks[cpos + 1..].trim_start();
    }

    (name_without_nicks, nicks)
}

/// Find the positions of the `open` and `close` characters, if they exist in the `haystack` such
/// that `open` occurs before `close`.
fn find_nickname_start_end(haystack: &str, open: char, close: char) -> Option<(usize, usize)> {
    if let Some(opos) = haystack.find(open) {
        // Found the opening character. look for the closing character after it
        if let Some(cpos) = haystack[opos + 1..].find(close) {
            // Found the closing character after the opening
            Some((opos, opos + 1 + cpos))
        } else {
            None
        }
    } else {
        None
    }
}

/// Checks if `part` is an initial, defined as an alphabetic character optionally followed by a
/// period.
fn is_initial(part: &str) -> bool {
    let chars = part.chars().collect::<Vec<char>>();

    match &chars[..] {
        [c] if c.is_alphabetic() => true,
        [c, '.'] if c.is_alphabetic() => true,
        _ => false,
    }
}

/// Checks if `part` is a prefix, defined in `PREFIXES`. The check is case-insensitive and ignores
/// trailing periods.
fn is_prefix(part: &str) -> bool {
    let normalized: String = part.trim_end_matches('.').to_lowercase();
    !is_initial(&normalized[..]) && PREFIXES.iter().any(|prefix| prefix == &normalized)
}

/// Checks if `part` is a suffix, defined in `SUFFIXES`. The check is case-insensitive and ignores
/// trailing periods.
fn is_suffix(part: &str) -> bool {
    let normalized: String = part
        .chars()
        .filter(|c| c != &'.')
        .collect::<Vec<char>>()
        .into_iter()
        .collect::<String>()
        .to_lowercase();
    !is_initial(&normalized[..]) && SUFFIXES.iter().any(|suffix| suffix == &normalized)
}

/// Checks if `part` is a title, defined in `SUFFIXES`. The check is case-insensitive and ignores
/// trailing periods.
fn is_title(part: &str) -> bool {
    let normalized: String = part.trim_end_matches('.').to_lowercase();
    TITLES.contains(&&normalized[..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_position() {
        let jfk = "John (Jack) Kennedy";
        let (opos, cpos) = find_nickname_start_end(jfk, '(', ')').unwrap();

        assert_eq!(5, opos);
        assert_eq!(10, cpos);
    }

    #[test]
    fn extracts_nickname() {
        let jfk = "John (Jack) Kennedy";

        // Opening and closing positions of nickname delimiters
        let (opos, cpos) = find_nickname_start_end(jfk, '(', ')').unwrap();
        assert_eq!(5, opos);
        assert_eq!(10, cpos);

        let (full, nicks) = extract_nicknames(jfk);
        assert_eq!(full, "John Kennedy");
        assert_eq!(nicks.len(), 1);
        assert_eq!(nicks.get(0).unwrap(), "Jack");
    }

    #[test]
    fn extracts_titles() {
        let p = PersonName::parse("Rev. Dr. Martin Luther King Jr.").unwrap();

        assert_eq!(p.title, "Rev. Dr.");
        assert_eq!(p.first, "Martin");
        assert_eq!(p.middle, "Luther");
        assert_eq!(p.last, "King");
        assert_eq!(p.suffix, "Jr.");
        assert_eq!(p.nicknames, Vec::<String>::new());
    }

    #[test]
    fn join_on_conjunctions() {
        let mut p = PersonName::new();

        let input = "mr and mrs smith"
            .split(' ')
            .map(|s| s.to_string())
            .collect();
        let results = p.join_on_conjunctions(&input);

        assert_eq!(p.title, "mr and mrs");
        assert_eq!(results, &["mr and mrs", "smith"]);
    }

    #[test]
    fn nickname_only() {
        let p = PersonName::parse("(jimbo)").unwrap();

        assert_eq!(p.title, "");
        assert_eq!(p.first, "");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "");
        assert_eq!(p.suffix, "");
        assert_eq!(p.nicknames, vec!["jimbo"]);
    }

    #[test]
    fn format1() {
        let p = PersonName::parse("mrs jane q public").unwrap();

        assert_eq!(p.title, "mrs");
        assert_eq!(p.first, "jane");
        assert_eq!(p.middle, "q");
        assert_eq!(p.last, "public");
        assert_eq!(p.suffix, "");
        assert_eq!(p.nicknames, Vec::<String>::new());
    }

    #[test]
    fn format2() {
        let p = PersonName::parse("Mr. john smith, esq").unwrap();

        assert_eq!(p.title, "Mr.");
        assert_eq!(p.first, "john");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "smith");
        assert_eq!(p.suffix, "esq");
        assert_eq!(p.nicknames, Vec::<String>::new());
    }

    #[test]
    fn format3() {
        let p = PersonName::parse("johnson, john (johnny), iii").unwrap();

        assert_eq!(p.title, "");
        assert_eq!(p.first, "john");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "johnson");
        assert_eq!(p.suffix, "iii");
        assert_eq!(p.nicknames, vec!["johnny"]);
    }

    #[test]
    fn last_name_prepositions() {
        let p = PersonName::parse("Guido van Rossum").unwrap();
        println!("{:?}", p);

        assert_eq!(p.first, "Guido");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "van Rossum");
        assert_eq!(p.suffix, "");

        let p2 = PersonName::parse("van Rossum, Guido").unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn last_name_prepositions2() {
        let p = PersonName::parse("Johannes Diderik van der Waals").unwrap();
        println!("{:?}", p);

        assert_eq!(p.first, "Johannes");
        assert_eq!(p.middle, "Diderik");
        assert_eq!(p.last, "van der Waals");
        assert_eq!(p.suffix, "");
    }

    #[test]
    fn equality() {
        let p1 = PersonName::parse("john x smith").unwrap();
        let p2 = PersonName::parse("smith, john x").unwrap();

        assert_eq!(p1, p2);
    }

    #[test]
    fn mr_johnson() {
        let p = PersonName::parse("Mr. Johnson").unwrap();

        assert_eq!(p.title, "Mr.");
        assert_eq!(p.first, "");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "Johnson");
    }

    #[test]
    fn profession() {
        let p = PersonName::parse("Marie Curie, Ph.D.").unwrap();
        assert_eq!(p.first, "Marie");
        assert_eq!(p.middle, "");
        assert_eq!(p.last, "Curie");
        assert_eq!(p.suffix, "Ph.D.");
    }
}
