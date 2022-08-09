#![feature(byte_slice_trim_ascii)]
use std::fs::File;
use std::io::{self, Read};
use std::str;
use std::io::BufReader;

const REQUIRED_3_1_KEYWORDS: [&str; 16] = [
    "$BEGINANALYSIS", // byte-offset to the beginning of analysis segment
    "$BEGINDATA", // byte-offset of beginning of data segment
    "$BEGINSTEXT", // byte-offset to beginning of text segment
    "$BYTEORD", // byte order for data acquisition computer
    "$DATATYPE", // type of data in data segment (ASCII, int, float)
    "$ENDANALYSIS", // byte-offset to end of analysis segment
    "$ENDDATA", // byte-offset to end of data segment
    "$ENDSTEXT", // byte-offset to end of text segment
    "$MODE", // data mode (list mode - preferred, histogram - deprecated)
    "$NEXTDATA", // byte-offset to next data set in the file
    "$PAR", // number of parameters in an event
    "$PnB", // number of bits reserved for parameter number n
    "$PnE", // amplification type for parameter n
    "$PnN", // short name for parameter n
    "$PnR", // range for parameter n
    "$TOT" // total number of events in the data set
];

const OPTIONAL_3_1_KEYWORDS: [&str; 2] = [
    "$ABRT", // events lost due to acquisition electronic coincidence
    "$BTIM"
];
pub struct Header {
    pub fcs_version: String,
    pub txt_start: u32,
    pub txt_end: u32,
    pub data_start: u32,
    pub data_end: u32,
    pub analysis_start: u32,
    pub analysis_end: u32,
}

pub struct Metadata {
    pub fcs_version: String,
    pub begins_supplemental_txt: u32,
    pub ends_supplemental_txt: u32

}

/*
pub struct Analysis {
    pub gating: [i32]
}
*/

/*
pub struct FlowData {
    header: Header
    //metadata: &'a Metadata<'a>,
    //data: &'a [f64],
    //analysis: &'a Analysis
}
*/

// read fcs file
pub fn read_fcs(file_name: &str) -> Result<(), io::Error> {
    let file = File::open(&file_name)?;
    let header = read_header(&file);
    //let metadata = read_metadata(header.txt_start, header.txt_end);
    //let data = read_data(header.data_start, header.data_end);
    //let analysis = read_analysis(header.analysis_start, header.analysis_end);
    
    /*
    return FlowData{
        header,
        //metadata: &metadata,
        //data: &data,
        //analysis: &analysis
    }
    */

    Ok(())
}

pub fn read_header(file: &File) -> Result<Header, io::Error> {
    let mut reader = BufReader::new(file);

    // FCS verison
    let mut fcs_version: [u8; 6] = [0; 6];
    reader.by_ref().take(6).read(&mut fcs_version)?;
    let fcs_version = str::from_utf8(&mut fcs_version)
        .expect("Invalid FCS header: Error reading FCS version");

    // spaces
    let mut space_buffer: [u8; 4] = [0; 4];
    reader.by_ref().take(4).read(&mut space_buffer)?;
    let converted_byte = str::from_utf8(&mut space_buffer)
        .expect("Unable to convert byte to space string");
    if converted_byte != "    " {
        panic!("Invalid FCS header: Error reading spaces between sections");
    }

    // text, data, & analysis offsets
    let mut offsets: [u32; 6] = [0; 6];
    let mut _buffer: [u8; 8] = [0; 8];

    for i in 0..6 {
        reader.by_ref().take(8).read(&mut _buffer)?;
        let trimmed_buffer = _buffer.trim_ascii();
        let offset_str = str::from_utf8(&trimmed_buffer)
            .expect("Unable to convert ASCII whitespace trimmed buffer to str");
        offsets[i] = offset_str.parse::<u32>()
            .expect("Unable to convert string to u32");
    }

    Ok(Header{
        fcs_version: fcs_version.to_string(),
        txt_start: offsets[0],
        txt_end: offsets[1],
        data_start: offsets[2],
        data_end: offsets[3],
        analysis_start: offsets[4],
        analysis_end: offsets[5]
    })
}

/*
pub fn read_metadata(file: &File, start: u32, end: u32) -> Result<Metadata, io::Error> {
    println!("{} {}", start, end);

    Ok(Metadata{
        parameter_names: vec!["fitc".to_string()]
    })
}
*/

/*
pub fn read_data(start: i32, end: i32) -> [f64] {
    println!("{} {}", start, end);
    let data = [0.0, 0.0];

    return data
}
*/

/*
pub fn read_analysis(start: i32, end: i32) -> &'static Analysis {
    println!("{} {}", start, end);

    return &Analysis{
        gating: [1]

    }
}
*/


