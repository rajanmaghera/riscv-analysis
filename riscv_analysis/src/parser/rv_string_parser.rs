use super::{EmptyFileReader, ParseError, ParserNode, RVParser};

/// A simplified parser to read a string into `ParserNodes`, for testing.
pub struct RVStringParser;

impl RVStringParser {
    /// Parse a string representation into a list of `ParserNodes` and `ParseErrors`,
    /// for test purposes.
    ///
    /// This is a top-level function that is used to parse RISC-V assmebly text
    /// into parser nodes. It is a wrapper for `RVParser` and should only be
    /// used for test purposes, as it does not handle file reading.
    ///
    /// ```
    /// use riscv_analysis::parser::{RVStringParser, ParserNode};
    /// let (nodes, errors) = RVStringParser::parse_from_text("add x1, x10, x11\n");
    /// assert_eq!(nodes.len(), 2);
    /// assert_eq!(errors.len(), 0);
    /// matches!(&nodes[0], ParserNode::ProgramEntry(_));
    /// matches!(&nodes[1], ParserNode::Arith(_));
    /// assert_eq!(nodes[1].to_string(), "add ra <- a0, a1");
    /// ```
    #[must_use]
    pub fn parse_from_text(text: &str) -> (Vec<ParserNode>, Vec<ParseError>) {
        let mut parser = RVParser::new(EmptyFileReader::new(text));
        parser.parse_from_file(EmptyFileReader::get_file_path(), false)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn can_parse_from_text() {
        let (nodes, errors) = RVStringParser::parse_from_text("add x1, x10, x11\n");
        assert_eq!(nodes.len(), 2);
        assert_eq!(errors.len(), 0);
        matches!(&nodes[0], ParserNode::ProgramEntry(_));
        matches!(&nodes[1], ParserNode::Arith(_));
        assert_eq!(nodes[1].to_string(), "add ra <- a0, a1");
    }

    #[test]
    fn can_emit_parse_errors() {
        let (nodes, errors) =
            RVStringParser::parse_from_text("add x1, x10, x11\nadd x1, x10, x11\njall");
        assert_eq!(nodes.len(), 3);
        assert_eq!(errors.len(), 1);
        matches!(&nodes[0], ParserNode::ProgramEntry(_));
        matches!(&nodes[1], ParserNode::Arith(_));
        matches!(&nodes[2], ParserNode::Arith(_));
        matches!(&errors[0], ParseError::UnexpectedToken(_));
    }

    #[test]
    fn can_emit_error_on_include_directive() {
        let (nodes, errors) = RVStringParser::parse_from_text(".include \"file.s\"");
        assert_eq!(nodes.len(), 1);
        assert_eq!(errors.len(), 1);
        matches!(&nodes[0], ParserNode::Directive(_));
        matches!(&errors[0], ParseError::FileNotFound(_));
    }

    #[test]
    fn can_emit_error_on_self_reference() {
        let text = format!(".include \"{}\"\n", EmptyFileReader::get_file_path());
        let (nodes, errors) = RVStringParser::parse_from_text(&text);
        assert_eq!(nodes.len(), 1);
        assert_eq!(errors.len(), 1);
        matches!(&nodes[0], ParserNode::Directive(_));
        matches!(&errors[0], ParseError::FileNotFound(_));
    }
}
