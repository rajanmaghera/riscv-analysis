#[cfg(test)]
mod tests {
    use crate::parser::{RVInstructionNode, RVStringParser};

    #[test]
    fn can_parse_comments_without_errors() {
        let parser_output =
            RVStringParser::parse_from_text("add x1, x10, x11 #this is my comment\n");
        assert_eq!(parser_output.nodes.len(), 1);
        assert_eq!(parser_output.errors.len(), 0);
        matches!(&parser_output.nodes[0], RVInstructionNode::Arith(_));
        assert_eq!(parser_output.nodes[0].to_string(), "add ra <- a0, a1");
    }
}
