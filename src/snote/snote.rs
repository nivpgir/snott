use std::{error::Error, path::Path};
use std::str::FromStr;

use chumsky::{Parser, prelude::Simple};
// use chumsky::error::Error;

use super::{SNoteSection, snote};




struct SNote{
    raw_content: String,
    sections: Vec<SNoteSection>
}


#[cfg(test)]
mod tests{
    use std::{env::temp_dir, str::FromStr};

    use super::SNote;

    #[test]
    fn create_snote() {
	SNote::new();
    }
    #[test]
    fn create_snote_from_string() {
	assert!(SNote::from_str("").is_ok());
    }

    #[test]
    fn save_to_file(){
	let output_file = temp_dir().with_file_name("temp_note.snot");
	SNote::from_str("").unwrap()
	    .save_to(&output_file).unwrap();

	assert!(output_file.exists())
    }

    #[test]
    fn update_contents(){
	let mut note = SNote::from_str("").unwrap();
	let new_content = "* headline";
	note.set_raw(new_content);

	assert_eq!(note.raw_content, new_content);
    }
}
impl SNote {
    pub(crate) fn new() -> Self{
	Self {
	    raw_content: Default::default(),
	    sections: snote().parse("").unwrap_or_default()
	}
    }

    pub(crate) fn save_to(&self, output_file: &Path) -> Result<(), Box<dyn Error>> {
	std::fs::write(output_file, &self.raw_content)?;
	Ok(())
    }

    pub(crate) fn set_raw(&mut self, new_content: impl AsRef<str>) {
        self.raw_content = new_content.as_ref().to_string()
    }
}


impl FromStr for SNote{
    type Err=String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sections = snote().parse(s)
	    .map_err(|e|e.into_iter()
		     .reduce(chumsky::Error::merge)
		     .unwrap_or_else(|| Simple::custom(0..0, "unknown"))
		     .to_string())?;
	Ok(Self {
	    sections,
	    raw_content: s.to_string()
	})
    }
}
