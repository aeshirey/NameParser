/// Predefined titles that generally occur before a given name.
/// For example, "Dr." in "Dr. Martin Luther King, Jr."
/// Titles here are specified in lowercase and without trailing periods.
pub static TITLES: [&str; 9] = [
    "rev", "sir", "madam", "miss", "misses", "dr", "doctor", "mr", "mrs",
];

/// Predefined suffixes that generally occur after a surname.
/// For example, "Jr." in "Dr. Martin Luther King, Jr."
pub static SUFFIXES: [&str; 12] = [
    "jr", "sr", "iii", "iv",

    // professional suffixes:
    "esq", "esquire", "rn", "lpn", "cpa", "md", "phd", "dds",
];

/// Predefined conjunctions that join components prior to parsing.
/// For example, "Mr.", "and", "Mrs." become "Mr. and Mrs." and are saved as the title.
/// "John", "and", "Jane" become "John and Jane" and are saved as the first.
pub static CONJUNCTIONS: [&str; 3] = ["&", "and", "y"];

/// Predefined surname prefixes that are part of the surname.
/// For example, "van" in "Guido van Rossum", parsing out to be first:"Guido", last:"van Rossum".
pub static PREFIXES: [&str; 10] = [
    "de", "del", "du", "la", "der", "van", "st", "ste", "vel", "von",
];
