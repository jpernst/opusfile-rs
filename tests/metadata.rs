

extern crate opusfile;

use std::fs;
use std::path::Path;


#[test]
fn metadata ()
{
    let mut file = fs::File::open(Path::new(".").join("tests").join("bach.opus")).unwrap();
    let opus = opusfile::OggOpusFile::from_read(&mut file).unwrap();

    let head = opus.head(None).unwrap();
    let tags = opus.tags(None).unwrap();

    assert_eq!(tags.user_comments.len(), 9);
    for &(ref tag, ref data) in tags.user_comments.iter() {
        let tag : &str = tag;
        match &*tag {
            "ALBUM" => assert_eq!(data, "Goldberg Variations, BWV 988"),
            "ARTIST" => assert_eq!(data, "Johann Sebastian Bach"),
            "DESCRIPTION" => assert_eq!(data, "From the Musopen Collection"),
            "ENCODER" => assert_eq!(data, "opusenc from opus-tools 0.1.9"),
            "ENCODER_OPTIONS" => assert_eq!(data, "--bitrate 48"),
            "GENRE" => assert_eq!(data, "Classical"),
            "TITLE" => assert_eq!(data, "Variation 4"),
            "TRACKNUMBER" => assert_eq!(data, "05"),
            "TRACKTOTAL" => assert_eq!(data, "31"),
            _ => panic!("Unexpected tag"),
        }
    }
}


