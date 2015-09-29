

extern crate opusfile;

use std::io;
use std::fs;
use std::path::Path;


fn main ()
{
    let mut file = fs::File::open(Path::new(".").join("tests").join("bach.opus")).unwrap();
    let opus = opusfile::OggOpusFile::from_read(&mut file).unwrap();

    let head = opus.head(None).unwrap();
    let tags = opus.tags(None).unwrap();

    println!("{}", tags.user_comments.len());
    for (tag, data) in tags.user_comments {
        println!("{}: {}", tag, data);
    }
}


