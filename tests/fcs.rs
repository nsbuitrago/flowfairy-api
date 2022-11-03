use flowfairy_api::read_fcs;
use std::io;

const FORMAT_3_0_TESTFILE: &str = "/Users/nsbuitrago/Dev/flowfairy-api/tests/format_3_0.fcs";

#[test]
pub fn test_fcs_3_0_reader() -> Result<(), io::Error>{
    // read FCS 3.0
    let flowdata = read_fcs(FORMAT_3_0_TESTFILE)?;
    // check metadata
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
        "$BTIM", "$ETIM", "$CYT", "$CYTSN", "$DATE", "$FIL", "$TIMESTEP",
        "$TR"
    ];
    assert_eq!(flowdata.metadata.version, "FCS3.0");
    assert_eq!(flowdata.metadata.keywords, keywords);
    
    let total_events = flowdata.metadata.values.get("$TOT").unwrap().parse::<usize>().unwrap();
    let total_params = flowdata.metadata.values.get("$PAR").unwrap().parse::<usize>().unwrap();
    assert_eq!(total_events * total_params, flowdata.data.len());
    
    Ok(())
}


