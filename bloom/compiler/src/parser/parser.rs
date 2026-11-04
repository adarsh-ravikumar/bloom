use oxc_allocator::Allocator as OxcAllocator;

use crate::{
    analysis::PlatformNamespace,
    common::{EMPTY_SPAN, Source, Span, SpannedString},
    diagnostic::Diagnostic,
    module::{
        Expression, ExpressionId, Module, ModuleMemberType, ModuleMetadata,
    },
    parser::{ScriptAnalyzer, nodes::*},
    utils,
};

#[derive(Default)]
struct ParsedScript {
    open: Span,
    source: SpannedString,
}

pub struct Parser<'a> {
    // position will always point to the closest unread byte
    pub pos: usize,
    pub src: &'a Source,
    pub diagnostics: Vec<Diagnostic>,

    oxc_allocator: OxcAllocator,
    script_analyzer: ScriptAnalyzer,
    script: Option<ParsedScript>,
    module: &'a mut Module,

    // stacks for handling block-related diagnostics
    open_tags: Vec<TagName>,
    open_controls: Vec<Span>,
    last_closed_tag: TagName,
    last_closed_control: Span,

    // context variable handling
    context_vars: Vec<String>,

    // platform handling
    platform_parent: String,
    current_platform_tag_type: Option<PlatformNamespace>,
}

const COMMAND: &'static str = "command";
const EVENT: &'static str = "event";
const REACTIVE: &'static str = "reactive";
const RESERVED_TAGS: [&'static str; 2] = ["panel", "slot"];

impl<'a> Parser<'a> {
    pub fn new(src: &'a Source, module: &'a mut Module) -> Self {
        Self {
            pos: 0,
            src,

            oxc_allocator: OxcAllocator::new(),
            script_analyzer: ScriptAnalyzer::new(),
            script: None,
            module,

            diagnostics: Vec::new(),

            open_tags: Vec::new(),
            open_controls: Vec::new(),
            last_closed_tag: TagName::default(),
            last_closed_control: EMPTY_SPAN,

            context_vars: Vec::new(),

            platform_parent: String::default(),
            current_platform_tag_type: None,
        }
    }

    pub fn parse(&mut self) {
        let frag = self.parse_fragment(&[FragmentEnd::Root]);

        let mut metadata = ModuleMetadata::new();

        let script = self.script.take().unwrap_or_default();

        self.module.root = Some(frag);
        self.module.script = (
            script.open,
            self.preprocess_script(&mut metadata, script.source),
        );

        self.module.metadata = metadata;

        self.script_analyzer
            .analyze(&mut self.module, &self.oxc_allocator);
    }

    fn consume(&mut self, by: usize) {
        self.pos += by;
    }

    fn peek(&self, by: usize) -> u8 {
        self.src.get(self.pos + by)
    }

    fn eat<S: AsRef<str>>(&mut self, seq: S) -> bool {
        let seq = seq.as_ref().as_bytes();

        for (i, &ch) in seq.iter().enumerate() {
            if self.peek(i) != ch {
                return false;
            }
        }

        self.consume(seq.len());
        true
    }

    fn restore(&mut self, to: usize) {
        assert!(to <= self.pos);
        self.pos = to;
    }

    fn consume_whitespace(&mut self) {
        while matches!(self.peek(0), b' ' | b'\t' | b'\n') {
            self.consume(1);
        }
    }

    pub fn synchronize(&mut self, set: &[&'static str]) {
        loop {
            if self.peek(0) == 0 {
                return;
            }

            let pos = self.pos;

            for &item in set {
                if self.eat(item) {
                    self.restore(pos); // we restore, as we do not intend to eat the sync token
                    return;
                }
            }

            self.consume(1);
        }
    }

    fn parse_fragment(&mut self, expected_end: &[FragmentEnd]) -> Fragment {
        let mut nodes: Vec<FragmentNode> = Vec::new();
        let mut parts: Parts = Vec::new();

        loop {
            let cur = self.peek(0);

            match cur {
                0 => break,

                b'<' => {
                    if !parts.is_empty() {
                        nodes.push(FragmentNode::Parts(std::mem::take(
                            &mut parts,
                        )));
                    }

                    if self.eat("<!--") {
                        self.parse_comment();
                        continue;
                    }

                    if self.peek(1) == b'/' {
                        if expected_end.contains(&FragmentEnd::Tag) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end tag
                            // cursor points to the '<'
                            return Fragment { nodes };
                        } else {
                            self.handle_stray_closing_tag();
                            return Fragment { nodes };
                        }
                    }

                    // parse tag
                    let (tag, is_self_closing) = self.parse_open_tag();

                    if tag.name.namespace.is_none()
                        && tag.name.name.string == "script"
                    {
                        self.parse_script_tag(&tag.name)
                    } else {
                        nodes.push(self.parse_tag(tag, is_self_closing))
                    }
                }

                b'{' => {
                    if self.peek(1) == b'#' {
                        if !parts.is_empty() {
                            nodes.push(FragmentNode::Parts(std::mem::take(
                                &mut parts,
                            )));
                        }

                        nodes.push(self.parse_control());
                    } else if self.peek(1) == b'/' {
                        if !parts.is_empty() {
                            nodes.push(FragmentNode::Parts(std::mem::take(
                                &mut parts,
                            )));
                        }

                        if expected_end.contains(&FragmentEnd::ControlEnd) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            self.handle_stray_closing_control_block();
                            return Fragment { nodes };
                        }
                    } else if self.peek(1) == b':' {
                        if !parts.is_empty() {
                            nodes.push(FragmentNode::Parts(std::mem::take(
                                &mut parts,
                            )));
                        }

                        if expected_end.contains(&FragmentEnd::ControlCase) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            self.emit_stray_control_case();
                            self.synchronize(&["\n", "}"]);
                            self.eat("}");
                            continue;
                        }
                    } else {
                        let expr_start = self.pos;

                        self.consume(1);
                        let expr = self.parse_embedded_expression("}");

                        parts.push(Part::Expression(expr));

                        if !self.eat("}") {
                            self.emit_unclosed_expression(expr_start);
                        }
                    }
                }

                _ => {
                    // default to parse text
                    if let Some(text) = self.parse_text(&[b'<', b'{']) {
                        parts.push(Part::Text(text))
                    }
                }
            }
        }

        // flush parts
        if !parts.is_empty() {
            nodes.push(FragmentNode::Parts(parts));
        }

        Fragment { nodes }
    }

    fn parse_text(&mut self, delim: &[u8]) -> Option<String> {
        let start = self.pos;

        while !delim.contains(&self.peek(0)) && self.peek(0) != 0 {
            self.consume(1);
        }

        let is_empty = self.src.view(start, self.pos).trim().is_empty();

        if !is_empty {
            Some(self.src.view(start, self.pos).to_string())
        } else {
            None
        }
    }

    fn parse_comment(&mut self) {
        let comment_start = Span::new(self.pos - 4, self.pos); // <!--
        let mut nested_comments = 0usize;

        loop {
            if self.eat("<!--") {
                nested_comments += 1;
            }

            if self.eat("-->") {
                if nested_comments == 0 {
                    self.consume_whitespace();
                    return;
                } else {
                    nested_comments -= 1;
                }
            }

            if self.peek(0) == 0 {
                // comment was never closed
                self.emit_unclosed_comment(comment_start);
                return;
            }

            self.consume(1);
        }
    }

    fn parse_embedded_expression(&mut self, delim: &str) -> ExpressionId {
        let source = self.identify_guest_boundary(delim);
        let mut expr = Expression::new(source, self.context_vars.clone());

        println!("EXPRESSION: {expr:?}");

        self.script_analyzer
            .parse_expression(&mut expr, &self.oxc_allocator);

        self.module.insert_expression(expr)
    }

    fn identify_guest_boundary(&mut self, delim: &str) -> SpannedString {
        // goal of the method is to correctly find the end of a JS expression
        // expects that current pointer is at the start of the expr
        let mut num_braces = 0usize;
        let mut quote: Option<u8> = None;

        let start = self.pos;

        loop {
            let cur = self.peek(0);

            let may_restore_to = self.pos;
            if num_braces == 0 && quote.is_none() && self.eat(delim) {
                self.restore(may_restore_to);
                return if start == self.pos
                    || self.src.view(start, self.pos).trim().is_empty()
                {
                    SpannedString::default()
                } else {
                    let span = Span::new(start, self.pos);
                    SpannedString {
                        span,
                        string: self.src.view_span(span).to_string(),
                    }
                };
            }

            if cur == 0 {
                // we have exhausted the expression until EOF
                // this may or may not be intended but we do know that we are yet to find the delimiter.
                // this error is NOT a concern of the embedded scanner. the caller is expected to resolve it.
                return SpannedString::default();
            }

            let cur_is_quote = matches!(cur, b'"' | b'\'' | b'`');
            // we don't handle the template literal case separately.
            // the JS/TS parser can later take over for that.
            // the string handling guarantees that all the braces within the literal are skipped
            // and that is the only case we genuienly care about

            if cur_is_quote {
                if quote.is_none() {
                    quote = Some(cur);
                } else if quote == Some(cur) {
                    quote = None;
                }

                self.consume(1);
                continue;
            }

            if quote.is_some() && cur == b'\\' {
                self.consume(2); // skip whatever character comes after the escape
                // this is naive, but it handles \', \" and \`, which is more than sufficient
                continue;
            }

            // handle comments
            if cur == b'/' {
                if self.peek(1) == b'/' {
                    self.consume(2);

                    while !matches!(self.peek(0), b'\n' | b'\0') {
                        self.consume(1);
                    }

                    continue;
                } else if self.peek(1) == b'*' {
                    self.consume(2);

                    loop {
                        if self.peek(0) == b'\0' {
                            // TODO: diagnostics
                            println!("block comment was never closed");
                            return SpannedString::default();
                        }

                        if self.peek(0) == b'*' && self.peek(1) == b'/' {
                            self.consume(2);
                            break;
                        }

                        self.consume(1);
                    }

                    continue;
                }

                continue;
            }

            // handle braces
            if cur == b'{' && quote.is_none() {
                num_braces += 1;
            }

            if cur == b'}' && num_braces > 0 && quote.is_none() {
                num_braces -= 1;
            }

            self.consume(1);
        }
    }

    fn byte_to_str(byte: u8) -> &'static str {
        match byte {
            b' ' => " ",
            b'\t' => "\t",
            b'\n' => "\n",
            b'}' => "}",
            b'{' => "{",
            b'<' => "<",
            b'>' => ">",
            b'/' => "/",
            b'=' => "=",
            b':' => ":",
            b'"' => "\"",
            b'\'' => "'",
            _ => panic!(
                "byte_to_str called with unsupported byte '{byte}':{}",
                byte as char
            ),
        }
    }

    fn parse_ident(&mut self, delim: &[u8]) -> SpannedString {
        let start = self.pos;

        if !matches!(self.peek(0), b'a'..b'z' | b'A'..b'Z' | b'_') {
            let mut sync_set = delim
                .iter()
                .map(|&b| Self::byte_to_str(b))
                .collect::<Vec<&str>>();

            sync_set.extend(&[" ", "\t", "\n"]);

            self.synchronize(&sync_set);

            return SpannedString::default();
        }

        self.consume(1);

        loop {
            let cur = self.peek(0);

            if matches!(cur, b' ' | b'\t' | b'\n' | b'\0')
                || delim.contains(&cur)
            {
                break;
            }

            let is_valid_char = matches!(cur, b'a'..b'z' | b'A'..b'Z' | b'0'..b'9' | b'$' | b'-' | b'_');

            if !is_valid_char {
                let mut sync_set = delim
                    .iter()
                    .map(|&b| Self::byte_to_str(b))
                    .collect::<Vec<&str>>();

                sync_set.extend(&[" ", "\t", "\n"]);
                self.synchronize(&sync_set);

                return SpannedString::default();
            }

            self.consume(1);
        }

        let span = Span::new(start, self.pos);

        SpannedString {
            span,
            string: self.src.view_span(span).to_string(),
        }
    }

    fn parse_tag_name(&mut self) -> TagName {
        let name_start = self.pos; // record for diagnostics
        let mut tag_name = TagName::default();

        let mut namespace_prefix = self.parse_ident(&[b':', b'>', b'/']);
        let is_namespaced = self.eat(":");

        if self.current_platform_tag_type.is_none()
            && namespace_prefix.span == EMPTY_SPAN
        {
            self.platform_parent = tag_name.name.string.clone();

            self.current_platform_tag_type = Some(
                PlatformNamespace::from_string(tag_name.name.string.as_str()),
            )
        }

        // slots can be used with implicit namspace suffix (i.e without a suiffix) as well.
        // hence, we initialize it to default. if it is used with a suffix, is_namespaced is
        // already true, and the case is handled in the latter block.
        if namespace_prefix.string == "slot" {
            tag_name.namespace = Some(TagNamespace::Slot);
            tag_name.name = SpannedString::default();
        }

        if is_namespaced {
            if tag_name.namespace.is_none() {
                tag_name.namespace =
                    Some(if namespace_prefix.span == EMPTY_SPAN {
                        TagNamespace::Platform(
                            // current_platform_tag_type is guaranteed to be set by this point.
                            self.current_platform_tag_type
                                .as_ref()
                                .unwrap()
                                .clone(),
                        )
                    } else {
                        TagNamespace::Custom(namespace_prefix)
                    });
            }

            let name = self.parse_ident(&[b'>', b'/']);

            if name.span == EMPTY_SPAN {
                // TODO: diagnostics
                println!("expected tag name.");
            }

            tag_name.name = name
        }
        // we only absorb the name if the tag is NOT a slot. slots allow for empty names.
        else if !matches!(tag_name.namespace, Some(TagNamespace::Slot)) {
            if namespace_prefix.span == EMPTY_SPAN {
                self.emit_invalid_tag_name(name_start, self.pos);
            }

            // we aren't parsing a namespaced tag. hence, whatever we parsed to be the
            // namespace prefix IS the actual tag name.
            tag_name.name = std::mem::take(&mut namespace_prefix);
        }

        tag_name
    }

    fn parse_open_tag(&mut self) -> (Tag, bool) {
        let tag_open_pos = self.pos;
        self.eat("<");

        let tag_name = self.parse_tag_name();

        self.consume_whitespace();

        // attribute list
        let diag_pos_before_attribs = self.diagnostics.len();
        let attributes = self.parse_attributes();

        // self closing
        let self_closing = self.eat("/>");

        // we have an open tag.
        if !self_closing {
            self.open_tags.push(tag_name.clone());
        }

        if !self_closing && !self.eat(">") {
            // we failed to establish the opening-tag boundary.
            // attribute diagnostics were hence producced speculatively and can no longer be
            // trusted, because we don't know that we were actually parsing a well-delimitied
            // attribute list. remove all errors emitted by `parse_attrbiutes`.

            while self.diagnostics.len() > diag_pos_before_attribs {
                self.diagnostics.pop();
            }

            self.emit_expect_open_tag_end(tag_open_pos);
        }

        (
            Tag {
                name: tag_name,
                attributes,
                fragment: None,
            },
            self_closing,
        )
    }

    fn parse_attribute_value(&mut self) -> Option<AttributeValue> {
        let expr_start = self.pos;
        if self.eat("{") {
            let expr = self.parse_embedded_expression("}");

            if !self.eat("}") {
                self.emit_unclosed_expression(expr_start);
            }

            return Some(AttributeValue::Expression(expr));
        }

        let quote = self.peek(0);
        let quote_start = self.pos;

        if matches!(quote, b'\'' | b'"') {
            self.consume(1);

            // value must either be a quoted string or an expression
            let mut parts = Vec::new();

            while self.peek(0) != quote && self.peek(0) != 0 {
                let text = self.parse_text(&[quote, b'{']);
                if let Some(text) = text {
                    parts.push(Part::Text(text));
                }

                let expr_start = self.pos;

                if !self.eat("{") {
                    break;
                }

                let expr = self.parse_embedded_expression("}");

                if !self.eat("}") {
                    self.emit_unclosed_expression(expr_start);
                }

                parts.push(Part::Expression(expr));
            }

            let quote = if quote == b'"' { "\"" } else { "'" };

            if !self.eat(quote) {
                self.emit_unclosed_attribute_value(quote_start);
            }

            return Some(AttributeValue::Parts(parts));
        }

        return None;
    }

    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        // attributes can be of one of the following kinds
        // name
        // name={}
        // name="..."
        // name='...'
        // {...}

        while !matches!(self.peek(0), b'>' | b'<' | b'/') && self.peek(0) != 0 {
            self.consume_whitespace();
            // {...}
            let expr_start = self.pos;
            if self.eat("{") {
                let expr = self.parse_embedded_expression("}");

                if !self.eat("}") {
                    self.emit_unclosed_expression(expr_start);
                }

                let name = self.module.get_expression(expr).source.clone();

                attributes.push(Attribute::Regular(RegularAttribute {
                    name,
                    value: AttributeValue::Expression(expr),
                }));

                self.consume_whitespace();
                continue;
            }

            // ident
            let start = self.pos; // record for diagnostics
            let name = self.parse_ident(&[b'=', b'{', b'>', b'"', b'\'', b':']);

            if name.span == EMPTY_SPAN {
                self.emit_invalid_attribute_name(start, self.pos);
            }

            self.consume_whitespace();

            // binding
            if self.eat(":") {
                self.consume_whitespace();

                let bind_from = name;

                let bind_to = self
                    .parse_ident(&[b'=', b'{', b'>', b'"', b'\'', b':', b'/']);

                let bind_to = if bind_to.span == EMPTY_SPAN {
                    bind_from.clone()
                } else {
                    bind_to
                };

                self.consume_whitespace();

                let value = if self.eat("=") {
                    // if there is a binding value, we consume it.
                    if let Some(v) = self.parse_attribute_value() {
                        Some(v)
                    } else {
                        // invalid value
                        // diagnostics would have already been pushed. do nothing here.
                        continue;
                    }
                } else {
                    None
                };

                attributes.push(Attribute::Binding(BindingAttribute {
                    bind_from,
                    bind_to,
                    value,
                }));

                self.consume_whitespace();
                continue;
            }

            // boolean
            if !self.eat("=") {
                attributes.push(Attribute::Regular(RegularAttribute {
                    name,
                    value: AttributeValue::True,
                }));

                self.consume_whitespace();
                continue;
            }

            // value
            let value = self.parse_attribute_value();
            if let Some(value) = value {
                attributes
                    .push(Attribute::Regular(RegularAttribute { name, value }));
            } else {
                // invalid value
                // diagnostics would have already been pushed. do nothing here.
                continue;
            }

            self.consume_whitespace();
        }

        attributes
    }

    fn parse_close_tag(&mut self, matching_name: &TagName) {
        let tag_start = self.pos;

        // this is a defensive grammar assertion.
        // this case should ideally NEVER occur. but if it does, that means the parser has failed
        // to validate the closing conditions, and hence we must expect garbage.
        if !self.eat("</") {
            self.emit_expected_close_tag();
            self.synchronize(&[">", "\n", "<"]);
            return;
        }

        let tag_name = self.parse_tag_name();

        if !matching_name.is_name_match(&tag_name) {
            // look for candidate
            let mut candidate_exists = false;

            for (mut i, open) in self.open_tags.iter().rev().enumerate() {
                if tag_name.is_name_match(open) {
                    candidate_exists = true;
                    self.last_closed_tag = tag_name.clone();

                    // we have found a match. now, we can safely pop all the tags
                    // we pop until we hit i = 0, which means the tag at the top
                    // is the currently matched tag
                    while i >= 1 {
                        self.open_tags.pop();
                        i -= 1;
                    }

                    break;
                }
            }

            if candidate_exists {
                self.emit_unclosed_tag(matching_name.span(), tag_name.span());
                self.restore(tag_start);
                return;
            } else {
                self.emit_mismatched_tag(matching_name.span(), tag_name.span());
            }
        }

        // pop the currently open tag
        self.open_tags.pop();

        if self.last_closed_tag.is_name_match(&tag_name) {
            self.last_closed_tag = TagName::default();
        }

        if !self.eat(">") {
            self.emit_expect_close_tag_end(Span::new(tag_start, self.pos));
        }
    }

    fn parse_tag(
        &mut self,
        mut tag: Tag,
        is_self_closing: bool,
    ) -> FragmentNode {
        // register dependencies
        match tag.name.namespace {
            Some(TagNamespace::Custom(_)) => {
                // TODO: add support for custom namespaces
                todo!("Custom namespaces not supported");
            }

            None => {
                let key = &tag.name.name.string;

                if !self.module.dependencies.contains_key(key)
                    && !RESERVED_TAGS.contains(&key.as_str())
                {
                    self.module
                        .dependencies
                        .insert(key.clone(), tag.name.span());
                }
            }

            _ => (),
        }

        if is_self_closing {
            return FragmentNode::Tag(tag);
        }

        let fragment = self.parse_fragment(&[FragmentEnd::Tag]);

        tag.fragment = Some(fragment);

        // if we have hit EOF before trying to parse the close tag,
        // the current tag was never closed
        if self.peek(0) == 0 {
            if self.last_closed_tag.span().start > tag.name.span().start {
                self.emit_unclosed_tag(
                    tag.name.span(),
                    self.last_closed_tag.span(),
                );
            } else {
                self.emit_unclosed_tag_eof(tag.name.span());
            }

            return FragmentNode::Tag(tag);
        }

        // we will look for a closing tag only if the open_tags stack is non-empty
        // and the current top of the stack is equal to tag.name (compare TagName, including spans)
        // this is done because IF the stary-close-block handler deems us closed in order to recover,
        // we needn't look for a closing tag.
        if !self.open_tags.is_empty()
            && self.open_tags.last().unwrap() == &tag.name
        {
            self.parse_close_tag(&tag.name);
        }

        FragmentNode::Tag(tag)
    }

    fn handle_stray_closing_tag(&mut self) {
        // if when we encounter this stray closing tag and there is a possilbe
        // candidate in the open_tags stack that might consume it, then we
        // conclude that this is NOT a stray, and in fact belongs to the candidate.
        // this case occurs only when there is an unclosed control block as a child of
        // an widget
        // `
        // <div>
        //   {#if a == b}
        // </div>
        // `
        // recovery will simply be to restore to where the closing starts and
        // continue parsing. this will allow for the parent to parse it correctly
        //
        // as for the unclosed control block, we can simply report it's status as unclosed,
        // and pop it from the open stack. recursion will take naturally handle
        // multiple such unclosed control blocks.
        //
        // the stary case is always guaranteed to occur before we get to the
        // mismatched control block end case. both the cases do the exact same
        // pop-the-stack and push-the-match algorithm. when processing the stray,
        // the error-free case is the desired block is found on the top of the stack.

        // cursor sits at '<' expecting to read '</...>'
        let restore_pos = self.pos;

        self.eat("</");

        self.consume_whitespace();

        let closing_name = self.parse_tag_name();
        let closing_span = closing_name.span();

        // we will synchronize to either '>' or a whitespace
        self.synchronize(&[">", " ", "\n", "\t"]);

        // find a candidate in the open_tags stack
        let mut candidate_exists = false;

        while !self.open_tags.is_empty() {
            let open = self.open_tags.pop().unwrap();

            if closing_name.is_name_match(&open) {
                candidate_exists = true;

                self.last_closed_tag = closing_name;

                // we push it back to the open_controls stack as the actual control
                // block to which the tag belongs to will consume it
                self.open_tags.push(open);
                break;
            }
        }

        if candidate_exists {
            // we have found a match. we can now report the unclosed control block,
            // pop it, and restore the cursor
            let open = self.open_controls.pop().unwrap();
            self.emit_unclosed_control(open, closing_span);
            self.restore(restore_pos);
        } else {
            // no match. actual stray.
            self.emit_stray_closing_tag();
            self.synchronize(&["\n", ">"]);
            self.eat(">");
        }
    }

    fn handle_stray_closing_control_block(&mut self) {
        // if when we encounter this stray closing control block and there is a possilbe
        // candidate in the open_controls stack that might consume it, then we
        // conclude that this is NOT a stray, and in fact belongs to the candidate.
        // this case occurs only when there is an unclosed tag as a child of
        // a control block
        // `
        // {#if a == b}
        //      <div>
        // {/if}
        // `
        // recovery will simply be to restore to where the closing starts and
        // continue parsing. this will allow for the parent to parse it correctly
        //
        // as for the unclosed tag, we can simply report it's status as unclosed,
        // and pop it from the open stack. recursion will take naturally handle
        // multiple such unclosed tags.
        //
        // the stary case is always guaranteed to occur before we get to the
        // mismatched control block end case. both the cases do the exact same
        // pop-the-stack and push-the-match algorithm. when processing the stray,
        // the error-free case is the desired block is found on the top of the stack.

        // cursor sits at '{' expecting to read '{/...}'
        let restore_pos = self.pos;

        self.eat("{/");

        self.consume_whitespace();

        let closing_name_start = self.pos;

        // we will synchronize to either '}' or a whitespace
        self.synchronize(&["}", " ", "\n", "\t"]);

        let closing_name = Span::new(closing_name_start, self.pos);

        // find a candidate in the open_controls stack
        let mut candidate_exists = false;

        while !self.open_controls.is_empty() {
            let open = self.open_controls.pop().unwrap();

            if self.src.view_span(closing_name) == self.src.view_span(open) {
                candidate_exists = true;
                self.last_closed_control = closing_name;

                // we push it back to the open_controls stack as the actual control
                // block to which the tag belongs to will consume it
                self.open_controls.push(open);
                break;
            }
        }

        if candidate_exists {
            // we have found a match. we can now report the unclosed tag, pop it,
            // and restore the cursor
            let open = self.open_tags.pop().unwrap();
            self.emit_unclosed_tag(open.span(), closing_name);
            self.restore(restore_pos);
        } else {
            // no match. actual stray.
            self.emit_stray_closing_block();
            self.synchronize(&["\n", "}"]);
            self.eat("}");
        }
    }

    fn parse_control(&mut self) -> FragmentNode {
        let start = self.pos;

        self.eat("{#");

        self.consume_whitespace();

        // the control block only cares about whitespace.
        // parse_ident handles whitespace internally.
        // b' ' is passed here to keep the API simple
        // rather than introducing an Option<u8>
        let name = self.parse_ident(&[b' ']);
        if name.span == EMPTY_SPAN {
            self.emit_invalid_control_name(name.span.start, name.span.end);
        }

        self.open_controls.push(name.span);

        let control_block = match name.string.as_str() {
            "if" => self.parse_control_if(start, name.span),
            "each" => self.parse_control_each(start),
            _ => {
                // unknown control block
                unreachable!()
            }
        };

        // if we have hit EOF before trying to parse the close block,
        // the current block was never closed
        if self.peek(0) == 0 {
            if self.last_closed_control.start > name.span.start {
                self.emit_unclosed_control(name.span, self.last_closed_control);
            } else {
                self.emit_unclosed_control_eof(name.span);
            }

            return FragmentNode::Control(control_block);
        }

        // we will look for a closing of the block only if the open_controls stack is non-empty
        // and the current top of the stack is equal to control_block.name (compare spans' start and end)
        // this is done because IF the stary-close-tag handler deems us closed in order to recover,
        // we needn't look for a closing block.
        if !self.open_controls.is_empty()
            && self.open_controls.last().unwrap() == &name.span
        {
            self.parse_control_block_close(name.span);
        }

        FragmentNode::Control(control_block)
    }

    fn is_valid_control_terminator<S: AsRef<str>>(&mut self, name: S) -> bool {
        let name = name.as_ref();

        let start = self.pos;

        let is_valid = self.eat(name)
            && (self.eat("}") || self.eat("\t") || self.eat("\n"));

        if is_valid {
            self.restore(self.pos - 1);
        } else {
            self.restore(start);
        }

        is_valid
    }

    fn parse_control_block_close(&mut self, name: Span) {
        let close_start = self.pos;

        // this is a defensive grammar assertion.
        // this case should ideally NEVER occur. but if it does, that means the parser has failed
        // to validate the closing conditions, and hence we must expect garbage.
        if !self.eat("{/") {
            self.emit_expected_close_tag();
            self.synchronize(&["}", "\n", "{"]);
            return;
        }

        self.consume_whitespace();

        let closing_name_start = self.pos;
        let is_valid_closing =
            self.is_valid_control_terminator(self.src.view_span(name));

        if is_valid_closing {
            self.eat("}");
            self.open_controls.pop();

            if self.last_closed_control
                == Span::new(closing_name_start, self.pos)
            {
                self.last_closed_control = EMPTY_SPAN;
            }

            return;
        }

        // ending does NOT match.
        self.synchronize(&["}", " ", "\n"]);
        let closing_name = Span::new(closing_name_start, self.pos);

        self.consume_whitespace();

        // look for candidate
        let mut candidate_exists = false;

        while !self.open_controls.is_empty() {
            let open = self.open_controls.pop().unwrap();

            if self.src.view_span(closing_name) == self.src.view_span(open) {
                candidate_exists = true;
                self.last_closed_control = closing_name;

                // we push it back to the open_tags stack as the actual widget
                // to which the tag belongs to will consume it
                self.open_controls.push(open);
                break;
            }
        }

        if candidate_exists {
            self.emit_unclosed_control(name, closing_name);
            self.restore(close_start);
            return;
        } else {
            self.emit_mismatched_control(name, closing_name);
        }

        // pop the currently open tag
        self.open_controls.pop();

        if self.last_closed_control == closing_name {
            self.last_closed_control = EMPTY_SPAN;
        }

        if !self.eat("}") {
            // TODO: error
        }
    }

    fn parse_control_if(
        &mut self,
        control_start: usize,
        name: Span,
    ) -> Control {
        let control_start = Span::new(control_start, self.pos);

        self.consume_whitespace();

        let expr_start = self.pos;

        let expr = self.parse_embedded_expression("}");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            // expression was not closed
            self.emit_unclosed_expression(expr_start);
        }

        let span = self.module.get_expression(expr).source.span;

        if span == EMPTY_SPAN {
            self.emit_expected_control_expression(control_start, "if");
        }

        if !self.eat("}") {
            self.emit_unclosed_control_header(control_start, "if"); // reaches fatal EOF
        }

        let fragment = self.parse_fragment(&[
            FragmentEnd::ControlCase,
            FragmentEnd::ControlEnd,
        ]);

        // while we have {:, we have to parse elif, else
        let mut elifs = Vec::new();
        let mut else_body = None;
        let mut else_start = EMPTY_SPAN;

        loop {
            let closing_start = self.pos;

            if self.eat("{/") {
                // revert and return
                self.restore(closing_start);
                break;
            }

            // the breaking conditions when we do not terminate through the {/ path
            // is either we have exhausted the source and hit EOF, or we were closed
            // by the stray analyzer, in which case, we will no longer be on the top of
            // the open_controls stack.
            let is_eof = self.peek(0) == 0;
            let is_on_top = !self.open_controls.is_empty()
                && self.open_controls.last().unwrap() == &name;

            if is_eof || !is_on_top {
                break;
            }

            let case_start = self.pos;
            let is_case = self.eat("{:");

            self.consume_whitespace();

            if is_case {
                if self.is_valid_control_terminator("else") {
                    else_start = Span::new(case_start, self.pos);
                    else_body = Some(self.parse_else(else_start));
                    continue;
                }

                if self.eat("elif ") {
                    if else_body.is_some() {
                        self.emit_case_after_else(else_start, case_start);
                        // no sync needed. we will continue parsing
                    }

                    elifs
                        .push(self.parse_elif(Span::new(case_start, self.pos)));
                    continue;
                }

                // invalid case
                self.synchronize(&[" ", "\n"]); // sync to possible end of case name
                self.emit_unknown_control_case(case_start, &["elif", "else"]);

                // sync to the the start of the next case or a closing case
                self.synchronize(&["{:", "{/"]); // sync to possible end of case name
            }
        }

        Control::If(IfControl {
            condition: expr,
            body: fragment,
            elifs,
            else_body,
        })
    }

    fn parse_elif(&mut self, case_start: Span) -> ElifControl {
        self.consume_whitespace();

        let expr_start = self.pos;

        let expr = self.parse_embedded_expression("}");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            // expression was not closed
            self.emit_unclosed_expression(expr_start);
        }

        let span = self.module.get_expression(expr).source.span;

        if span == EMPTY_SPAN {
            self.emit_expected_control_expression(case_start, "elif");
        }

        if !self.eat("}") {
            self.emit_unclosed_control_header(case_start, "elif"); // reaches fatal EOF
        }

        let fragment = self.parse_fragment(&[
            FragmentEnd::ControlCase,
            FragmentEnd::ControlEnd,
        ]);

        ElifControl {
            condition: expr,
            body: fragment,
        }
    }

    fn parse_else(&mut self, case_start: Span) -> Fragment {
        self.consume_whitespace();

        // we are to NOT accept any conditions here
        // we expect the } to occur right after {:else
        // what we can try, is naively sync to the nearest '}', and report that
        // if there isn't any, then we can simply

        if !self.eat("}") {
            self.synchronize(&["}", "\n"]);

            // if we DID find a }, then we say "did not expect expression"
            // otherwise, we just report missing '}'
            if self.eat("}") {
                let expr_end = self.pos - 1;
                self.emit_unexpected_control_expression(
                    case_start,
                    Span::new(case_start.end + 1, expr_end),
                    Span::new(self.pos - 1, self.pos - 1),
                );
            } else {
                // TODO: The else could also mean we hit a new line, as that is
                // a valid sync point. But, the error points to EOF
                self.emit_unclosed_control_header(case_start, "else"); // reaches fatal EOF
            }
        }

        self.parse_fragment(&[
            FragmentEnd::ControlEnd,
            FragmentEnd::ControlCase, // this is an error state. but we still track it for recovery
        ])
    }

    fn parse_control_each(&mut self, control_start: usize) -> Control {
        let control_start = Span::new(control_start, self.pos);

        self.consume_whitespace();

        let context_start = self.pos; // record for diagnostics
        let context = self.parse_ident(&[b' ']);
        if context.span == EMPTY_SPAN {
            self.emit_invalid_each_context_name(context_start, self.pos);
        }

        self.context_vars.push(context.string.clone());

        self.consume_whitespace();

        // key
        let expr_start = self.pos;
        let key = if self.eat("{") {
            let key = self.parse_embedded_expression("}");
            self.consume_whitespace();

            if self.peek(0) == 0 {
                // expression was not closed
                self.emit_unclosed_expression(expr_start);
            }

            self.eat("}");

            Some(key)
        } else {
            None
        };

        self.consume_whitespace();

        println!("{}, {}", self.pos, self.src.get(self.pos) as char);

        if !self.eat("in") {
            self.emit_expected_each_in(context.span);
        }

        self.consume_whitespace();

        if self.peek(0) == b'}' {
            self.emit_expected_control_expression(control_start, "each");
        }

        let expr_start = self.pos;

        let expr = self.parse_embedded_expression("}");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            // expression was not closed
            self.emit_unclosed_expression(expr_start);
        }

        if !self.eat("}") {
            self.emit_unclosed_control_header(control_start, "each"); // reaches fatal EOF
        }

        let body = self.parse_fragment(&[FragmentEnd::ControlEnd]);

        self.context_vars.pop();

        Control::Each(EachControl {
            expression: expr,
            context,
            key,
            body,
        })
    }

    fn parse_script_tag(&mut self, open: &TagName) {
        if let Some(script) = &self.script {
            self.emit_multiple_script_tags(script.open, open.span());
        }

        if let Some(tag) = self.open_tags.last()
            && tag != open
        {
            self.emit_nested_script_tag(open.span(), tag.span());
        }

        if let Some(&control) = self.open_controls.last() {
            self.emit_nested_script_tag(open.span(), control);
        }

        let source = self.identify_guest_boundary("</");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            self.emit_unclosed_tag_eof(open.span()); // reach fatal eof
        }

        self.parse_close_tag(open);

        self.script = Some(ParsedScript {
            open: open.span(),
            source,
        });
    }

    fn preprocess_script(
        &mut self,
        metadata: &mut ModuleMetadata,
        source: SpannedString,
    ) -> String {
        // "#" ("command" | "event" | "reactive") (space | tab)* <newline>
        // "function" (space | tab | newline)*  <identifier> [<...> | (...)]
        //
        // we do NOT perform error recovery in the preprocessor, as the syntax itself is
        // insufficient to render clear boundaries for synchronization, and it is added
        // complexity for a process that will eitherway end up terminating the compiler
        // without moving on to the next stage.
        //
        // the implementation itself is quite "hacky" but is intentional.

        let mut pos = source.span.start;

        let mut quote = None;

        let mut cur = self.src.get(pos);

        let mut spans = Vec::new();

        let mut script_consume_pointer = pos;
        let script_start = pos;

        // check for EOF = redundancy. this case should NEVER occur
        while cur != 0 && pos < source.span.end {
            let cur_is_quote = matches!(cur, b'"' | b'\'' | b'`');
            // we don't handle the template literal case separately.
            // the JS/TS parser can later take over for that.
            // the string handling guarantees that all the braces within the literal are skipped
            // and that is the only case we genuienly care about

            if cur_is_quote {
                if quote.is_none() {
                    quote = Some(cur);
                } else if quote == Some(cur) {
                    quote = None;
                }

                pos += 1;
                cur = self.src.get(pos);
                continue;
            }

            if quote.is_some() && cur == b'\\' {
                pos += 2;
                // this is naive, but it handles \', \" and \`, which is more than sufficient
                cur = self.src.get(pos);
                continue;
            }

            if cur == b'#' && quote.is_none() {
                let directive_start = pos;

                if script_consume_pointer < directive_start {
                    spans.push(Span::new(
                        script_consume_pointer,
                        directive_start,
                    ))
                }

                pos += 1;

                // skip whitespace
                while [b' ', b'\t', b'\n'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // now we try to consume till the next whitespace
                let start = pos;

                while !([b' ', b'\t', b'\n'].contains(&self.src.get(pos))) {
                    pos += 1
                }

                let end = pos;

                let directive = self.src.view(start, end);

                if !matches!(directive, COMMAND | EVENT | REACTIVE) {
                    self.emit_preprocessor_unknown_directive(
                        directive_start,
                        pos,
                    );
                    break;
                }

                script_consume_pointer = pos;

                // skip whitespace but NOT newline
                while [b' ', b'\t'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // expect newline
                if self.src.get(pos) != b'\n' {
                    self.emit_preprocessor_expected_newline(pos, pos);
                    break;
                }

                pos += 1;

                // skip all whtiespace
                while [b' ', b' ', b'\t', b'\n'].contains(&self.src.get(pos)) {
                    pos += 1
                }
                // try consume "function"
                let start = pos;

                while !([b' ', b'\t', b'\n'].contains(&self.src.get(pos))) {
                    pos += 1
                }

                let end = pos;

                if self.src.view(start, end) != "function" {
                    self.emit_preprocessor_expected_function(start, end);
                    break;
                }

                // skip whitespace
                while [b' ', b'\t', b'\n'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // try consume function name
                let name_start = pos;

                while !([b'(', b'<', b' ', b'\t', b'\n']
                    .contains(&self.src.get(pos)))
                {
                    pos += 1
                }

                let name_end = pos;

                let span = Span::new(name_start, name_end);
                let fn_name = self.src.view_span(span).to_string();

                if fn_name.trim().is_empty() {
                    self.emit_preprocessor_expected_function_name(
                        name_start, name_end,
                    );
                }

                let canonical_name =
                    utils::normalize_case(&fn_name, utils::Case::SnakeCase);

                println!("canonical name for {fn_name} is {canonical_name}");

                metadata.insert(
                    // canonical_name,
                    fn_name,
                    match directive {
                        COMMAND => ModuleMemberType::Command(None),
                        EVENT => ModuleMemberType::Event(None),
                        REACTIVE => ModuleMemberType::Reactive(None),
                        _ => panic!("unknown directive"),
                    },
                );
            }

            pos += 1;
            cur = self.src.get(pos);
        }

        if script_consume_pointer < source.span.end {
            spans.push(Span::new(script_consume_pointer, source.span.end))
        }

        // strip
        let mut stripped_script = String::new();

        for span in spans {
            let start = span.start - script_start;
            let end = span.end - script_start;

            stripped_script.push_str(&source.string[start..end]);
        }

        stripped_script
    }
}
