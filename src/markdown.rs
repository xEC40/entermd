use regex::Regex;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
enum Block {
    Paragraph(String),
    List(Vec<(usize, String)>),
    Header(u8, String),
    Code(String),
    Hr,
}

enum CurrentBlock {
    Paragraph(Vec<String>),
    List(Vec<(usize, String)>),
    Code(Vec<String>),
}

struct SplitState {
    blocks: Vec<Block>,
    current: Option<CurrentBlock>,
}

pub fn markdown_to_html(markdown: &str) -> String {
    let blocks = split_blocks(markdown);
    blocks.into_iter()
        .map(parse_block)
        .collect::<Vec<_>>()
        .join("\n") + "\n"
}

fn split_blocks(markdown: &str) -> Vec<Block> {
    let lines = markdown.lines().map(|s| s.trim_end()).collect::<Vec<_>>();
    let initial_state = SplitState {
        blocks: Vec::new(),
        current: None,
    };
    let mut state = lines.into_iter().fold(initial_state, |state, line| {
        split_blocks_reducer(state, line)
    });

    // Handle remaining current block
    if let Some(current) = state.current.take() {
        match current {
            CurrentBlock::Paragraph(lines) => {
                let content = lines.join("\n");
                state.blocks.push(Block::Paragraph(content));
            }
            CurrentBlock::List(items) => {
                state.blocks.push(Block::List(items));
            }
            CurrentBlock::Code(_) => unreachable!(),
        }
    }

    state.blocks
}

fn split_blocks_reducer(state: SplitState, line: &str) -> SplitState {
    let line = line.trim_end();

    // Handle code blocks first
    if line.starts_with("```") {
        if let Some(CurrentBlock::Code(lines)) = state.current {
            // Close code block
            let content = lines.join("\n");
            let mut new_blocks = state.blocks;
            new_blocks.push(Block::Code(content));
            return SplitState {
                blocks: new_blocks,
                current: None,
            };
        } else {
            // Open code block
            return SplitState {
                blocks: state.blocks,
                current: Some(CurrentBlock::Code(Vec::new())),
            };
        }
    }

    if let Some(CurrentBlock::Code(mut lines)) = state.current {
        lines.push(line.to_string());
        return SplitState {
            blocks: state.blocks,
            current: Some(CurrentBlock::Code(lines)),
        };
    }

    // empty lines handler
    if line.is_empty() {
        let mut new_blocks = state.blocks;
        let current = state.current;
        if let Some(current_block) = current {
            match current_block {
                CurrentBlock::Paragraph(lines) => {
                    new_blocks.push(Block::Paragraph(lines.join("\n")));
                }
                CurrentBlock::List(items) => {
                    new_blocks.push(Block::List(items));
                }
                CurrentBlock::Code(_) => unreachable!(),
            }
        }
        return SplitState {
            blocks: new_blocks,
            current: None,
        };
    }

    // handle horizontal rule
    lazy_static! {
        static ref HR_REGEX: Regex = Regex::new(r"^-{3,}$").unwrap();
    }
    if HR_REGEX.is_match(line) {
        let mut new_blocks = state.blocks;
        if let Some(current_block) = state.current {
            match current_block {
                CurrentBlock::Paragraph(lines) => {
                    new_blocks.push(Block::Paragraph(lines.join("\n")));
                }
                CurrentBlock::List(items) => {
                    new_blocks.push(Block::List(items));
                }
                CurrentBlock::Code(_) => unreachable!(),
            }
        }
        new_blocks.push(Block::Hr);
        return SplitState {
            blocks: new_blocks,
            current: None,
        };
    }

    // handle headers
    lazy_static! {
        static ref HEADER_REGEX: Regex = Regex::new(r"^(#{1,6})\s+(.*)").unwrap();
    }
    if let Some(caps) = HEADER_REGEX.captures(line) {
        let level = caps.get(1).unwrap().as_str().len() as u8;
        let text = caps.get(2).unwrap().as_str().to_string();
        let mut new_blocks = state.blocks;
        if let Some(current_block) = state.current {
            match current_block {
                CurrentBlock::Paragraph(lines) => {
                    new_blocks.push(Block::Paragraph(lines.join("\n")));
                }
                CurrentBlock::List(items) => {
                    new_blocks.push(Block::List(items));
                }
                CurrentBlock::Code(_) => unreachable!(),
            }
        }
        new_blocks.push(Block::Header(level, text));
        return SplitState {
            blocks: new_blocks,
            current: None,
        };
    }

    // handle list items
    lazy_static! {
        static ref LIST_REGEX: Regex = Regex::new(r"^(\s*)- (.+)$").unwrap();
    }
    if let Some(caps) = LIST_REGEX.captures(line) {
        let indent = caps.get(1).unwrap().as_str().len();
        let content = caps.get(2).unwrap().as_str().to_string();
        let mut new_state = state;

        // Check if current is not a list, finalize it
        if let Some(current_block) = new_state.current.take() {
            match current_block {
                CurrentBlock::List(_) => {
                    new_state.current = Some(current_block);
                }
                CurrentBlock::Paragraph(lines) => {
                    new_state.blocks.push(Block::Paragraph(lines.join("\n")));
                }
                CurrentBlock::Code(_) => unreachable!(),
            }
        }

        // add item to list
        if let Some(CurrentBlock::List(mut items)) = new_state.current {
            items.push((indent, content));
            new_state.current = Some(CurrentBlock::List(items));
        } else {
            new_state.current = Some(CurrentBlock::List(vec![(indent, content)]));
        }

        return new_state;
    }

    // handle other lines (paragraphs)
    let mut new_state = state;
    match new_state.current.take() {
        Some(CurrentBlock::Paragraph(mut lines)) => {
            lines.push(line.to_string());
            new_state.current = Some(CurrentBlock::Paragraph(lines));
        }
        Some(CurrentBlock::List(items)) => {
            // Close the list and start a new paragraph
            new_state.blocks.push(Block::List(items));
            new_state.current = Some(CurrentBlock::Paragraph(vec![line.to_string()]));
        }
        Some(CurrentBlock::Code(_)) => unreachable!(),
        None => {
            new_state.current = Some(CurrentBlock::Paragraph(vec![line.to_string()]));
        }
    }
    new_state
}

fn parse_block(block: Block) -> String {
    match block {
        Block::Paragraph(content) => parse_paragraph(&content),
        Block::List(items) => parse_list(items),
        Block::Header(level, text) => format!("<h{}>{}</h{}>", level, inline_parse(&text), level),
        Block::Code(content) => format!("<pre><code>{}</code></pre>", content),
        Block::Hr => "<hr>".to_string(),
    }
}

fn parse_paragraph(content: &str) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    if lines.len() >= 2 {
        if is_table_header(lines[0]) && is_table_separator(lines[1]) {
            let header_cells = parse_table_row(lines[0]);
            let separator_cells = parse_table_row(lines[1]);
            if header_cells.len() == separator_cells.len() {
                let body_lines = &lines[2..];
                if body_lines.iter().all(|line| is_table_row(line)) {
                    let body_rows: Vec<Vec<String>> = body_lines.iter().map(|line| parse_table_row(line)).collect();
                    if body_rows.iter().all(|row| row.len() == header_cells.len()) {
                        let mut html = vec!["<table>".to_string()];
                        html.push("  <thead>".to_string());
                        html.push("    <tr>".to_string());
                        for cell in header_cells {
                            html.push(format!("      <th>{}</th>", inline_parse(&cell)));
                        }
                        html.push("    </tr>".to_string());
                        html.push("  </thead>".to_string());
                        if !body_rows.is_empty() {
                            html.push("  <tbody>".to_string());
                            for row in body_rows {
                                html.push("    <tr>".to_string());
                                for cell in row {
                                    html.push(format!("      <td>{}</td>", inline_parse(&cell)));
                                }
                                html.push("    </tr>".to_string());
                            }
                            html.push("  </tbody>".to_string());
                        }
                        html.push("</table>".to_string());
                        return html.join("\n");
                    }
                }
            }
        }
    }
    format!("<p>{}</p>", inline_parse(content))
}

fn is_table_header(line: &str) -> bool {
    let cells = parse_table_row(line);
    !cells.is_empty() && cells.iter().all(|cell| !cell.trim().is_empty())
}

fn is_table_separator(line: &str) -> bool {
    lazy_static! {
        static ref SEPARATOR_REGEX: Regex = Regex::new(r"^:?-{3,}:?$").unwrap();
    }
    let cells = parse_table_row(line);
    !cells.is_empty() && cells.iter().all(|cell| SEPARATOR_REGEX.is_match(cell))
}

fn parse_table_row(line: &str) -> Vec<String> {
    line.trim().split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_table_row(line: &str) -> bool {
    line.trim().contains('|')
}

fn parse_list(items: Vec<(usize, String)>) -> String {
    if items.is_empty() {
        return "<ul></ul>".to_string();
    }

    //let min_indent = items.iter().map(|(indent, _)| *indent).min().unwrap();
    let unique_indents: HashSet<usize> = items.iter().map(|(indent, _)| *indent).collect();
    let mut unique_indents: Vec<usize> = unique_indents.into_iter().collect();
    unique_indents.sort();

    let indent_levels: HashMap<usize, usize> = unique_indents.iter()
        .enumerate()
        .map(|(i, &indent)| (indent, i))
        .collect();

    let mut result = Vec::new();
    let mut current_level = -1i32;
    let mut stack = Vec::new();

    for (i, (indent, content)) in items.iter().enumerate() {
        let normalized_indent = indent_levels[indent] as i32;

        if normalized_indent > current_level {
            for level in current_level + 1..=normalized_indent {
                let indent_str = "  ".repeat(level as usize);
                result.push(format!("{}<ul>", indent_str));
                stack.push(level);
            }
        } else if normalized_indent < current_level {
            while let Some(&top_level) = stack.last() {
                if top_level > normalized_indent {
                    stack.pop();
                    let indent_str = "  ".repeat(top_level as usize);
                    result.push(format!("{}</li>", indent_str));
                    result.push(format!("{}</ul>", indent_str));
                } else {
                    break;
                }
            }
        }

        if i > 0 && normalized_indent == current_level {
            let indent_str = "  ".repeat(current_level as usize);
            result.push(format!("{}</li>", indent_str));
        }

        let indent_str = "  ".repeat(normalized_indent as usize);
        result.push(format!("{}<li>{}", indent_str, inline_parse(content)));

        current_level = normalized_indent;
    }

    while let Some(level) = stack.pop() {
        let indent_str = "  ".repeat(level as usize);
        result.push(format!("{}</li>", indent_str));
        result.push(format!("{}</ul>", indent_str));
    }

    result.join("\n")
}

fn inline_parse(text: &str) -> String {
    let text = text.replace("--", "-");

    // handle images 
    // also handle resizing of image
    lazy_static! {
        static ref IMAGE_SIZE_REGEX: Regex = Regex::new(r"!\[(.*?)\]\((.*?)\)\{(.*?)\}").unwrap();
    }
    let text = IMAGE_SIZE_REGEX.replace_all(&text, |caps: &regex::Captures| {
        let alt = caps.get(1).unwrap().as_str();
        let url = caps.get(2).unwrap().as_str();
        let attrs = caps.get(3).unwrap().as_str();

        let mut img_tag = format!("<img src=\"{}\" alt=\"{}\"", url, alt);

        // extract width height
        lazy_static! {
            static ref WIDTH_REGEX: Regex = Regex::new(r"width=(\d+)").unwrap();
            static ref HEIGHT_REGEX: Regex = Regex::new(r"height=(\d+)").unwrap();
        }
        if let Some(cap) = WIDTH_REGEX.captures(attrs) {
            img_tag.push_str(&format!(" width=\"{}\"", &cap[1]));
        }
        if let Some(cap) = HEIGHT_REGEX.captures(attrs) {
            img_tag.push_str(&format!(" height=\"{}\"", &cap[1]));
        }

        img_tag.push('>');
        img_tag
    });

    // handle line breaks
    lazy_static! {
        static ref LINE_BREAK_REGEX: Regex = Regex::new(r"\\(\r\n|\n|\r)").unwrap();
    }
    let text = LINE_BREAK_REGEX.replace_all(&text, "<br>\n");

    // Other inline elements
    let replacements = [
        (Regex::new(r"\*\*(.+?)\*\*").unwrap(), r"<strong>$1</strong>"),
        (Regex::new(r"\*(.+?)\*").unwrap(), r"<em>$1</em>"),
        (Regex::new(r"~~(.+?)~~").unwrap(), r"<s>$1</s>"),
        (Regex::new(r"!\[(.*?)\]\((.*?)\)").unwrap(), "<img src=\"$2\"alt=\"$1\">"),
        (Regex::new(r"\[(.*?)\]\((.*?)\)").unwrap(), "<a href=\"$2\">$1</a>"),
        (Regex::new(r"`(.+?)`").unwrap(), r"<code>$1</code>"),
    ];

    let mut parsed = text.to_string();
    for (regex, replacement) in replacements.iter() {
        parsed = regex.replace_all(&parsed, *replacement).to_string();
    }

    parsed
}
