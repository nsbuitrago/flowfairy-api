use flowfairy_api::{read_header, read_metadata};
use std::collections::HashMap;
use std::io;
use std::fs::File;
use std::str;

const FORMAT_3_1_TESTFILE: &str = "/Users/nsbuitrago/Dev/flowfairy-api/tests/format_3_1.fcs";
// const FORMAT_2_1_TESTFILE: &str = "Users/nsbuitrago/Dev/flowfairy-api-tests/format_2_1.fcs";

#[test]
pub fn test_fcs_reader() -> Result<(), io::Error>{
    let file = File::open(&FORMAT_3_1_TESTFILE)?;
    
    // read FCS 3.1 header
    let header = read_header(&file)?;
    assert_eq!(header.fcs_version, "FCS3.1");
    assert_eq!(header.txt_start, 64);
    assert_eq!(header.txt_end, 8255);
    assert_eq!(header.data_start, 8256);
    assert_eq!(header.data_end, 445255);
    assert_eq!(header.analysis_start, 0);
    assert_eq!(header.analysis_end, 0);
    // read FCS 3.1 txt segment
    let text_segment = read_metadata(&file, header.txt_start, header.txt_end)?;
    let keywords = vec![
        "$BEGINANALYSIS", "$ENDANALYSIS", "$BEGINSTEXT", "$ENDSTEXT", 
        "$BEGINDATA", "$ENDDATA", "$MODE", "$DATATYPE", 
        "$BYTEORD", "$PAR", "$NEXTDATA", "$TOT", 
        "$P1N", "$P1B", "$P1E", "$P1R", "$P1S", 
        "$P2N", "$P2B", "$P2E", "$P2R", "$P2S",
        "$P3N", "$P3B", "$P3E", "$P3R", "$P3S",
        "$P4N", "$P4B", "$P4E", "$P4R", "$P4S",
        "$P5N", "$P5B", "$P5E", "$P5R", "$P5S",
        "$P6N", "$P6B", "$P6E", "$P6R", "$P6S",
        "$P7N", "$P7B", "$P7E", "$P7R", "$P7S", 
        "$P8N", "$P8B", "$P8E", "$P8R", "$P8S", 
        "$P9N", "$P9B", "$P9E", "$P9R", "$P9S", 
        "$P10N", "$P10B", "$P10E", "$P10R", "$P10S", 
        "$BTIM", "$ETIM", "$SPILLOVER", "$CYT", 
        "$CYTSN", "$DATE", "$FIL", "$TIMESTEP", "$TR"
    ];
    assert_eq!(text_segment.keywords, keywords);
    Ok(())
}

