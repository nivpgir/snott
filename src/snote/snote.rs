use std::convert::Infallible;
use std::str::FromStr;

use chumsky::Parser;
// use chumsky::error::Error;

use super::{SNoteSection, snote};




#[derive(Debug)]
pub struct SNote{
    pub raw_content: String,
    sections: Vec<SNoteSection>
}


#[cfg(test)]
mod tests{
    use std::str::FromStr;

    use crate::snote::SNoteSection;

    use super::SNote;

    #[test]
    fn failed_parsing_creates_a_single_paragraph() {
	let content = "not a valid note because theres no headline\nlalala";
	let note = SNote::from_str(content).unwrap();
	assert_eq!(note.sections, vec![SNoteSection::Paragraph(0..content.len())])
    }

    #[test]
    fn create_snote() {
	SNote::new();
    }
    #[test]
    fn create_snote_from_string() {
	assert!(SNote::from_str("").is_ok());
    }

    #[test]
    fn update_contents(){
	let mut note = SNote::from_str("").unwrap();
	assert_eq!(&note.raw_content, "");

	let new_content = "* headline";
	note = note.set_raw(new_content);
	assert_eq!(&note.raw_content, new_content);

    }
}
impl SNote {
    pub(crate) fn new() -> Self{
	Self {
	    raw_content: Default::default(),
	    sections: snote().parse("").unwrap_or_default()
	}
    }

    pub(crate) fn set_raw(mut self, new_content: impl AsRef<str>) -> Self{
        self.raw_content = new_content.as_ref().to_string();
	self.update_sections()
    }

    fn update_sections(mut self) -> Self{
	self.sections = snote()
	    .parse(self.raw_content.clone())
            .unwrap_or_else(|_| vec![SNoteSection::Paragraph(0..self.raw_content.len())]);
	self
    }
}


impl FromStr for SNote{
    type Err=Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
	Ok(Self {
	    sections: Default::default(),
	    raw_content: s.to_string()
	}.update_sections())
    }
}
