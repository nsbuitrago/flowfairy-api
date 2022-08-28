#![feature(byte_slice_trim_ascii)]
use core::panicking::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, BufReader, Seek, SeekFrom, BufRead};
use std::{str, vec};

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
    pub txt_start: u64,
    pub txt_end: u64,
    pub data_start: u64,
    pub data_end: u64,
    pub analysis_start: u64,
    pub analysis_end: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Parameter {
    name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub delimitter: u8,
    pub keywords: Vec<String>,
    pub kv: HashMap<String, String>
}

/*
pub struct Analysis {
    pub gating: [i32]
}
*/


// hold flow cytometry data
pub struct FlowData {
    pub metadata: Metadata,
    pub data: [f64],
    //analysis: &'a Analysis
}

// read fcs file
pub fn read_fcs(file_name: &str) -> Result<(), io::Error> {
    let file = File::open(&file_name)?;
    let header = read_header(&file)?;
    let metadata = read_metadata(&file, header.txt_start, header.txt_end)?;
    let data_start = metadata.kv.get("$BEGINDATA");
    let data_end = metadata.kv.get("$ENDDATA");
    let byte_ord = metadata.kv.get("$BYTEORD");
    let data_type = metadata.kv.get("$DATATYPE");
    let num_params = metadata.kv.get("$PAR");
    let num_events = metadata.kv.get("$TOT");
    
    //let data = read_data(header.data_end);
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
    let mut offsets: [u64; 6] = [0; 6];
    let mut _buffer: [u8; 8] = [0; 8];

    for i in 0..6 {
        reader.by_ref().take(8).read(&mut _buffer)?;
        let trimmed_buffer = _buffer.trim_ascii();
        let offset_str = str::from_utf8(&trimmed_buffer)
            .expect("Unable to convert ASCII whitespace trimmed buffer to str");
        offsets[i] = offset_str.parse::<u64>()
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

pub fn read_metadata(file: &File, start: u64, end: u64) -> Result<Metadata, io::Error> {
    let mut reader = BufReader::new(file);
    let mut text_segment: Metadata = Default::default();

    reader.seek(SeekFrom::Start(start))?;

    let mut _delimitter = [0u8;1];
    reader.read_exact(&mut _delimitter)?;
    text_segment.delimitter = _delimitter[0];
    let mut next_char = [0u8, 1];
    let mut partial_value: Vec<u8> = Vec::new();
    let mut previous_stream_position: u64;
    
    while reader.stream_position()? < end {
        let mut keyword: Vec<u8> = Vec::new();
        let mut value: Vec<u8> = Vec::new();
        // read keyword
        reader.read_until(text_segment.delimitter, &mut keyword)?;
        // read value
        loop {
            reader.read_until(text_segment.delimitter, &mut partial_value)?;
            value.append(&mut partial_value);
            previous_stream_position = reader.stream_position()?;
            reader.read_exact(&mut next_char)?;

            // if delimiter is not for escaping,
            // go to previous stream position and stop the value reading loop
            if next_char[0] != text_segment.delimitter {
                reader.seek(SeekFrom::Start(previous_stream_position))?;
                break;
            }
        }

        let keyword_value_pair = clean_keyword_value_pair(keyword, value, text_segment.delimitter)?;
        if keyword_value_pair.0 != "" {
            text_segment.keywords.push(keyword_value_pair.0);
            //text_segment.values(keyword_value_pair.0, keyword_value_pair.1);
        }
    }
    Ok(text_segment)
}

fn clean_keyword_value_pair(keyword: Vec<u8>, value: Vec<u8>, delimiter: u8) -> Result<(String, String), io::Error> {
    // check that last byte in keyword and value are delimiter character
    // FIXME: change this panic to a custom error type
    if keyword[keyword.len()-1] != delimiter || value[value.len()-1] != delimiter {
        panic!("File or parser may be corrupted")
    }
    // convert u8 vector to lossy utf8 string
    let keyword_lossy_str = str::from_utf8(&keyword[..keyword.len()-1]);
    let value_lossy_str = str::from_utf8(&value[..value.len()-1]);

    let keyword_valid_ascii = match keyword_lossy_str {
        Ok(keyword_lossy_str) => keyword_lossy_str,
        Err(_) => "",
    };

    let value_valid_ascii = match value_lossy_str {
        Ok(keyword_lossy_str) => keyword_lossy_str,
        Err(_) => "",
    };

    // trim leading and trailing whitespace
    let keyword_clean = keyword_valid_ascii.trim().to_string();
    let value_clean = value_valid_ascii.trim().to_string();

    return Ok((keyword_clean, value_clean))
}


/*
pub fn read_data(file: &File, data_start: u64, data_end: u64, mode: String, 
                 data_type: String, byte_ord: String, num_params: i64, 
                 num_events: i64) -> Result<Vec<f64>, io::Error> {
    // create reader
    let mut reader = BufReader::new(file);
    // go to start of data segment
    reader.seek(SeekFrom::Start(data_start))?;

    // make a u8 vector with capcity of data segment
    let buffer_capacity = data_end - data_start + 1;
    // initialize buffer
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_capacity as usize);
    // check that mode is not "L"
    if mode != String::from("L") {
        panic!("Only list mode supported as data mode");
    }

    // get the len of data
    let data_len: i64 = num_params * num_events;
    // read to buffer
    reader.read_exact(&mut buffer)?;
    let mut data = buffer.as_slice();

    match data_type.as_str() {
        "A" => panic!("ASCII type is deprecated in FCS 3.1"),
        "D" => {
            let data: Vec<f64> = vec![0.0; data_len as usize]; // make data vec with np*ne elements
            
        },
        "F" => {
            let data: Vec<f32> = vec![0.0, data_len as f32];
        },
        "I" => {
            panic!("Unimplemented I data type");
        }
    }

    Ok(data)
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


