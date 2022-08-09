use flowfairy_api::read_header;
use std::io;
use std::fs::File;

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
    //let metadata = read_metadata(&file, header.txt_start, header.txt_end)?;
    //let parameter_names = vec!["FITC", "SytoxBlue", "mScarlet"];

    //assert_eq!(metadata.parameter_names, parameter_names);

    // read FCS 3.1 data segment
    // read_data(&file, header.data_start, header.data_end);

    // read FCS 3.1 analysis segment
    // read_analysis(&file, header.analysis_start, header.analysis_end);

    Ok(())
}

