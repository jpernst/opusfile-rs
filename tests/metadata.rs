

extern crate opusfile;


static OPUS_DATA : &'static [u8] = include_bytes!("bach.opus");


#[test]
fn header ()
{
    let opus = opusfile::OggOpusFile::from_slice(OPUS_DATA).unwrap();
    let head = opus.head(None).unwrap();

    assert_eq!(head.mapping, vec![0, 1]);
    assert_eq!(head.version, 1);
    assert_eq!(head.pre_skip, 356);
    assert_eq!(head.input_sample_rate, 44100);
    assert_eq!(head.output_gain, 0);
    assert_eq!(head.mapping_family, 0);
    assert_eq!(head.channel_count, 2);
    assert_eq!(head.stream_count, 1);
    assert_eq!(head.coupled_count, 1);
}

#[test]
fn tags ()
{
    let opus = opusfile::OggOpusFile::from_slice(OPUS_DATA).unwrap();
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


