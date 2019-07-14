/// Indents every non-blank line in the input text by the given string.
pub fn indent(indentation: &str, text: String) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let lines2: Vec<String> = lines
        .iter()
        .map(|l| {
            if !l.is_empty() {
                format!("{}{}", indentation, l)
            } else {
                String::from(*l)
            }
        })
        .collect();
    lines2.join("\n")
}

#[cfg(test)]
mod indent {
    use super::*;

    #[test]
    fn empty_string() {
        let input = String::from("");
        let output = indent("    ", input);

        assert_eq!(output, "");
    }

    #[test]
    fn oneline_string() {
        let input = String::from("foo asdf bar");
        let output = indent("    ", input);

        assert_eq!(output, "    foo asdf bar");
    }

    #[test]
    fn multiline_string() {
        let input = String::from("foo\nasdf\nbar");
        let output = indent("    ", input);

        assert_eq!(output, "    foo\n    asdf\n    bar");
    }

    #[test]
    fn multiline_string_with_blankline() {
        let input = String::from("foo\n\nasdf\nbar");
        let output = indent("    ", input);

        assert_eq!(output, "    foo\n\n    asdf\n    bar");
    }
}
