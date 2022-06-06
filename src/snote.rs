use chumsky::{prelude::*, text::newline};

fn line_end() -> impl Parser<char, Option<char>, Error = Simple<char>> {
    newline().to(Some('\n')).or(end().to(None))

}

fn headline() -> impl Parser<char, SNote, Error = Simple<char>> {
    just("* ").ignore_then(take_until(line_end()))
	.map(|(headline, _)|
	     SNote::Headline(headline.iter().collect::<String>())
	)
}

fn newline_padding(amount: usize) -> impl Parser<char, SNote, Error = Simple<char>> {
    newline().repeated().at_least(amount)
	.map(|v|SNote::NewlinePadding(v.len()))
}

fn block() -> impl Parser<char, SNote, Error = Simple<char>> {
    take_until(newline_padding(2)
	       .or(newline_padding(1).then(end()).map(|(p, _)| p))
               .or(end().to(SNote::NewlinePadding(0))))
	.map(|(block, _)| SNote::Block(block.iter().collect::<String>()))
}

fn blocks() -> impl Parser<char, Vec<SNote>, Error = Simple<char>> {
    newline_padding(1).or_not().then(any().rewind().ignore_then(block())).repeated()
	.map(|padded_block|padded_block.into_iter().flat_map(|(padding, block)| {
	    if let Some(padding) = padding {
		vec![padding, block]
	    } else {
		vec![block]
	    }
	}).collect())
}

pub fn snote() -> impl Parser<char, Vec<SNote>, Error = Simple<char>> {
    headline()
	.then(blocks())
	.map(|(headline, mut blocks)| {blocks.insert(0, headline); blocks})
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SNote {
    NewlinePadding(usize),
    Block(String),
    Headline(String)
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::snote::{SNote, block, blocks, headline, newline_padding, snote};

    #[test]
    fn parse_block(){
	let paragraph = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz\n\n";
	let ast = block().parse(paragraph);
	eprintln!("{:?}", ast);
	assert!(ast.is_ok());
	assert_eq!(SNote::Block(paragraph.trim_end().to_string()),
		   ast.unwrap());
    }
    #[test]
    fn parse_a_file_ending_block(){
	let paragraph = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz";
	let ast = block().parse(paragraph);
	eprintln!("{:?}", ast);
	assert!(ast.is_ok());
	assert_eq!(SNote::Block(paragraph.trim_end().to_string()),
		   ast.unwrap());
    }

    #[test]
    fn parse_headline(){
	let the_headline = "headline";
	let ast = headline().parse(format!("* {}", the_headline));
	eprintln!("{:?}", ast);
	assert!(ast.is_ok());
	assert_eq!(SNote::Headline(the_headline.to_string()),
		   ast.unwrap());
    }

    #[test]
    fn parse_many_blocks(){
	let the_paragraph = "the\nfirst\nparagraph";
	let the_second_paragraph = "the second\n paragraph";
	let the_blocks = format!("{}\n\n{}\n\n", the_paragraph, the_second_paragraph);
	eprintln!("{}", &the_blocks);
	let ast = blocks().parse(the_blocks);
	assert!(ast.is_ok());
	assert_eq!(vec![
	    SNote::Block(the_paragraph.trim_end().to_string()),
	    SNote::Block(the_second_paragraph.trim_end().to_string()),
	], ast.unwrap());
    }
    #[test]
    fn parse_headline_and_paragraph(){
	let the_headline = "headline";
	let the_paragraph = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz\n\n";
	let note = format!("* {}\n{}", the_headline, the_paragraph);
	let ast = snote().parse(note);
	eprintln!("{:?}", ast);
	assert!(ast.is_ok());
	assert_eq!((
	    vec![
		SNote::Headline(the_headline.to_string()),
		SNote::Block(the_paragraph.trim_end().to_string()),
	    ]
	), ast.unwrap());
    }
    #[test]
    fn parse_headline_and_many_paragraphs(){
	let the_headline = "headline";
	let the_paragraph = "the\nfirst\nparagraph";
	let the_second_paragraph = "the second\n paragraph";
	let note = format!("* {}\n{}\n\n{}\n\n", the_headline, the_paragraph, the_second_paragraph);
	let ast = snote().parse(note);
	eprintln!("{:?}", ast);
	assert!(ast.is_ok());
	assert_eq!((
	    vec![
		SNote::Headline(the_headline.to_string()),
		SNote::Block(the_paragraph.trim_end().to_string()),
		SNote::Block(the_second_paragraph.trim_end().to_string()),
	    ]
	), ast.unwrap());
    }

    #[test]
    fn parse_padding(){
	assert_eq!(SNote::NewlinePadding(0), newline_padding(0).parse("").unwrap());
	assert_eq!(SNote::NewlinePadding(1), newline_padding(0).parse("\r\n").unwrap());
	assert_eq!(SNote::NewlinePadding(1), newline_padding(0).parse("\n").unwrap());
	assert_eq!(SNote::NewlinePadding(2), newline_padding(0).parse("\n\n").unwrap());
	assert_eq!(SNote::NewlinePadding(2), newline_padding(0).parse("\n\r\n").unwrap());
	assert_eq!(SNote::NewlinePadding(3), newline_padding(0).parse("\r\n\r\n\r\n").unwrap());
	assert_eq!(SNote::NewlinePadding(3), newline_padding(3).parse("\r\n\r\n\r\n").unwrap());
	assert!(newline_padding(4).parse("\r\n\r\n\r\n").is_err());
    }
}
