pub struct FormatContext<'a> {
    pub writer: &'a mut dyn std::fmt::Write,
    pub operand_context: OperandContext,
}

pub struct OperandContext {
    pub is_fp: bool,
}

impl std::fmt::Write for FormatContext<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer.write_str(s)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.writer.write_char(c)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.writer.write_fmt(args)
    }
}
