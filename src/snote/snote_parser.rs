use std::ops::Range;

use chumsky::{prelude::*, text::newline};

fn line_end() -> impl Parser<char, Option<char>, Error = Simple<char>> {
    newline().to(Some('\n')).or(end().to(None))
}

fn headline() -> impl Parser<char, SNoteSection, Error = Simple<char>> {
    just("* ")
        .ignore_then(take_until(line_end()))
        .map_with_span(|_, sp|
		       SNoteSection::Headline(sp.start..sp.end))
}

fn newlines_or_end(amount: usize) -> impl Parser<char, usize, Error = Simple<char>> {
    newline()
        .repeated()
	.at_least(1)
        .map(|s| Some(s.len()))
        .or(end().to(None))
        .try_map(move |nls, span|
		 match nls{
		     Some(nl_count) if nl_count >= amount => Ok(nl_count),
		     None => Ok(0),
		     Some(nl_count) =>
			 Err(Simple::custom(span, format!("not enough newlines ({})", nl_count)))
		 })
}
fn block() -> impl Parser<char, SNoteSection, Error = Simple<char>> {
    let block_separator = newlines_or_end(2).map_with_span(|_, s|s).rewind();
    any().rewind()
	.ignore_then(take_until(block_separator))
	.map_with_span(|(_, pad_span), sp| SNoteSection::Paragraph(sp.start..pad_span.end))
}

fn blocks() -> impl Parser<char, Vec<SNoteSection>, Error = Simple<char>> {
    block().map(|b|[b])
	.separated_by(newlines_or_end(2))
	.flatten()
}

pub fn snote() -> impl Parser<char, Vec<SNoteSection>, Error = Simple<char>> {
    headline().chain(
	blocks()
    ).padded().or_not().map(|m|m.unwrap_or_default())
}

type SnoteSpan = Range<usize>;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SNoteSection {
    Paragraph(SnoteSpan),
    Headline(SnoteSpan),
}


use SNoteSection::*;
impl SNoteSection {
    pub fn span(&self) -> Range<usize> {
	match self{
            Paragraph(sp) |
            Headline(sp) => sp.clone()
	}
    }
    pub fn content_span(&self) -> Range<usize> {
	match self{
            Paragraph(sp) |
            Headline(sp) => sp.clone()
	}
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use chumsky::Parser;

    use super::{SNoteSection, block, blocks, headline, newlines_or_end, snote};

    #[test]
    fn empty_snote_is_valid(){
	let (ast, _err) = snote().parse_recovery_verbose("");
	assert!(ast.is_some());
    }
    #[test]
    fn parse_block() {
        let paragraph = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz\n\n";
        let (ast, _err) = block().parse_recovery_verbose(paragraph);
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
        assert_eq!(SNoteSection::Paragraph(0..paragraph.len()), ast.unwrap());
    }
    #[test]
    fn parse_a_file_ending_block() {
        let paragraph = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz";
        let (ast, _err) = block().parse_recovery_verbose(paragraph);
        eprintln!("{:?}", _err);
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
        assert_eq!(SNoteSection::Paragraph(0..paragraph.len()), ast.unwrap());
    }

    #[test]
    fn parse_headline() {
        let the_headline = "headline";
        let (ast, _err) = headline().parse_recovery_verbose(format!("* {}", the_headline));
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
        assert_eq!(SNoteSection::Headline(0..the_headline.len()+2), ast.unwrap());
    }

    #[test]
    fn parse_many_blocks() {
        let the_paragraph = "the\nfirst\nparagraph";
        let the_second_paragraph = "the second\n paragraph";
        let the_third_paragraph = "the third\n paragraph";
	let (the_blocks, expected) = make_paragraphs([the_paragraph,
						      the_second_paragraph,
						      the_third_paragraph]);
        eprintln!("{}", &the_blocks);
        let (ast, _err) = blocks().parse_recovery_verbose(the_blocks);

	if !_err.is_empty() {dbg!(_err);}
        assert!(ast.is_some());
        assert_eq!(
            expected.into_iter().map(SNoteSection::Paragraph).collect::<Vec<_>>(),
            ast.unwrap()
        );
    }
    fn move_range(r: &Range<usize>, offset: usize) -> Range<usize>{
	(r.start+offset)..(r.end+offset)
    }
    #[test]
    fn parse_headline_and_paragraph() {
        let headline_str = "headline";
        let paragraph_str = "abcd efg hi h klmnop\nqrs tuv\nw\nx\nyz";
        let (note, expected) = make_snote(headline_str, [paragraph_str]);
        let (ast, _err) = snote().parse_recovery_verbose(note);
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
        assert_eq!(
            expected,
            ast.unwrap()
        );
    }

    #[test]
    fn parse_headline_and_many_paragraphs() {
        let the_headline = "headline";
        let the_paragraph = "the\nfirst\nparagraph";
        let the_second_paragraph = "the second\n paragraph";
        let (note, expected) = make_snote(the_headline, [the_paragraph, the_second_paragraph]);
        let (ast, _err) = snote().parse_recovery_verbose(note);
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
        assert_eq!(
            expected,
            ast.unwrap()
        );
    }
    #[test]
    fn snote_sections_are_adjacent() {
        let the_headline = "headline";
        let the_paragraph = "the\nfirst\nparagraph";
        let the_second_paragraph = "the second\n paragraph";
	let (s, expected_snot) = make_snote(the_headline, [the_paragraph, the_second_paragraph]);
	dbg!(&s);
        let (ast, _err) = snote().parse_recovery_verbose(s);
        eprintln!("{:?}", ast);
        assert!(ast.is_some());
	ast.as_ref().unwrap().windows(2)
	    .map(|sections|(sections[0].clone(), sections[1].clone()))
	    .for_each(|(s1, s2)|{
	    dbg!(&s1, &s2);
	    assert_eq!(s1.span().end, s2.span().start);
	});
        assert_eq!(
            expected_snot,
            ast.unwrap()
        );
    }

    fn make_paragraphs(paragraphs: impl IntoIterator<Item=impl AsRef<str>>)
		       -> (String, Vec<Range<usize>>){
	paragraphs.into_iter().scan(0, |l, p|{
	    let padded = format!("{}\n\n", p.as_ref());
	    let span = move_range(&(0..padded.len()), *l);
	    *l += padded.len();
	    Some((padded, span))
	}).unzip()
    }

    fn make_snote(headline: impl AsRef<str>, paragraphs: impl IntoIterator<Item=impl AsRef<str>>)
		  -> (String, Vec<SNoteSection>){
	let headline_text = format!("* {}\n", headline.as_ref());
	let headline_span = SNoteSection::Headline(0..headline_text.len());
	let (par_text, par_spans) = make_paragraphs(paragraphs);
	let spans = std::iter::once(headline_span).into_iter()
	    .chain(par_spans.into_iter()
		   .map(|sp|move_range(&sp, headline_text.len()))
		   .map(SNoteSection::Paragraph))
	    .collect();
	(headline_text + &par_text, spans)
    }

    #[test]
    fn parse_padding() {
        assert_eq!(
            0,
            newlines_or_end(0).parse("").unwrap()
        );
        assert_eq!(
            1,
            newlines_or_end(0).parse("\r\n").unwrap()
        );
        assert_eq!(
            1,
            newlines_or_end(0).parse("\n").unwrap()
        );
        assert_eq!(
            2,
            newlines_or_end(0).parse("\n\n").unwrap()
        );
        assert_eq!(
            2,
            newlines_or_end(0).parse("\n\r\n").unwrap()
        );
        assert_eq!(
            3,
            newlines_or_end(0).parse("\r\n\r\n\r\n").unwrap()
        );
        assert_eq!(
            3,
            newlines_or_end(3).parse("\r\n\r\n\r\n").unwrap()
        );
        assert!(newlines_or_end(4).parse("\r\n\r\n\r\n").is_err());
    }
}
