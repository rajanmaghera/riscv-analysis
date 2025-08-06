use super::{EmptyFileReader, RVParser, RVParserOutput};

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
    /// let parser_output = RVStringParser::parse_from_text("add x1, x10, x11\n");
    /// assert_eq!(parser_output.nodes.len(), 1);
    /// assert_eq!(parser_output.errors.len(), 0);
    /// matches!(&parser_output.nodes[0], ParserNode::Arith(_));
    /// assert_eq!(parser_output.nodes[0].to_string(), "add ra <- a0, a1");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an internal error.
    #[must_use]
    pub fn parse_from_text(text: &str) -> RVParserOutput {
        let mut parser = RVParser::new(EmptyFileReader::new(text));
        parser
            .parse_from_file(EmptyFileReader::get_file_path(), false)
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::{ParseError, ParserNode};

    #[test]
    fn can_parse_from_text() {
        let parser_output = RVStringParser::parse_from_text("add x1, x10, x11\n");
        assert_eq!(parser_output.nodes.len(), 1);
        assert_eq!(parser_output.errors.len(), 0);
        matches!(&parser_output.nodes[0], ParserNode::Arith(_));
        assert_eq!(parser_output.nodes[0].to_string(), "add ra <- a0, a1");
    }

    #[test]
    fn can_emit_parse_errors() {
        let parser_output =
            RVStringParser::parse_from_text("add x1, x10, x11\nadd x1, x10, x11\njall");
        assert_eq!(parser_output.nodes.len(), 2);
        assert_eq!(parser_output.errors.len(), 1);
        matches!(&parser_output.nodes[0], ParserNode::Arith(_));
        matches!(&parser_output.nodes[1], ParserNode::Arith(_));
        matches!(&parser_output.errors[0], ParseError::UnexpectedToken(_));
    }

    #[test]
    fn can_emit_error_on_include_directive() {
        let parser_output = RVStringParser::parse_from_text(".include \"file.s\"");
        assert_eq!(parser_output.nodes.len(), 0);
        assert_eq!(parser_output.errors.len(), 1);
        matches!(&parser_output.errors[0], ParseError::FileNotFound(_));
    }

    #[test]
    fn can_emit_error_on_self_reference() {
        let text = format!(".include \"{}\"\n", EmptyFileReader::get_file_path(),);
        let parser_output = RVStringParser::parse_from_text(&text);
        assert_eq!(parser_output.nodes.len(), 0);
        assert_eq!(parser_output.errors.len(), 1);
        matches!(&parser_output.errors[0], ParseError::FileNotFound(_));
    }
}
